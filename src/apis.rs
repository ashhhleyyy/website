use std::{
    fmt::Display,
    sync::Arc,
    time::{Duration, Instant},
};

use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder};
use serde::{de::DeserializeOwned, Deserialize};
use tokio::sync::{Mutex, RwLock};

use crate::error::Result;

const USER_AGENT: &str = "ashhhleyyy.dev website backend (v1, https://github.com/ashhhleyyy/)";
pub const PRONOUNS_PAGE_URL: &str = "https://en.pronouns.page/api/profile/get/ashhhleyyy?version=2&props=names&props=pronouns&props=flags&props=words";
pub const NOWPLAYING_URL: &str = "https://api.ashhhleyyy.dev/playing";
const MIN_REFRESH_TIME: Duration = Duration::from_secs(5);

pub(crate) mod fedi;
pub(crate) mod mediawiki;

pub(crate) static CLIENT: Lazy<Client> = Lazy::new(|| {
    ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()
        .expect("failed to build client")
});

#[derive(Clone)]
pub struct CachingFetcher<T: DeserializeOwned + Sized + Clone> {
    last_state: Arc<RwLock<T>>,
    last_updated: Arc<Mutex<Instant>>,
    url: String,
}

impl<T: DeserializeOwned + Sized + Clone> CachingFetcher<T> {
    pub async fn new(url: String) -> Result<Self> {
        let last_updated = Instant::now();
        let state = Self::fetch(&url).await?;
        Ok(Self {
            last_state: Arc::new(RwLock::new(state)),
            last_updated: Arc::new(Mutex::new(last_updated)),
            url,
        })
    }

    pub async fn get(&self) -> T {
        let now = Instant::now();
        let last_updated = self.last_updated.lock().await;
        if now - *last_updated > MIN_REFRESH_TIME {
            match Self::fetch(&self.url).await {
                Ok(profile) => {
                    *self.last_state.write().await = profile;
                }
                Err(e) => {
                    error!("failed to refresh data: {}", e);
                }
            }
        }
        self.last_state.read().await.clone()
    }

    async fn fetch(url: &str) -> Result<T> {
        let req = CLIENT.get(url).build()?;

        let res = CLIENT.execute(req).await?.error_for_status()?;

        let res = res.json::<T>().await?;

        Ok(res)
    }
}

#[derive(Clone, Deserialize)]
pub struct PronounsPageProfile {
    pub profiles: PronounsPageProfiles,
}

#[derive(Clone, Deserialize)]
pub struct PronounsPageProfiles {
    pub en: PronounsPageCard,
}

#[derive(Clone, Deserialize)]
pub struct PronounsPageCard {
    pub names: Vec<Word>,
    pub pronouns: Vec<Word>,
    pub flags: Vec<String>,
    pub words: Vec<Words>,
}

#[derive(Clone, Deserialize)]
pub struct Words {
    pub header: Option<String>,
    pub values: Vec<Word>,
}

#[derive(Clone, Deserialize)]
pub struct Word {
    pub value: String,
    pub opinion: WordOpinion,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WordOpinion {
    Yes,
    #[serde(rename = "meh")]
    Okay,
    #[serde(rename = "no")]
    Nope,
    Jokingly,
    #[serde(rename = "close")]
    OnlyClose,
}

impl WordOpinion {
    pub fn is_negative(&self) -> bool {
        match self {
            WordOpinion::Yes
            | WordOpinion::Jokingly
            | WordOpinion::OnlyClose
            | WordOpinion::Okay => false,
            WordOpinion::Nope => true,
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            WordOpinion::Yes => "💜",
            WordOpinion::Jokingly => "😛",
            WordOpinion::OnlyClose => "🧑‍🤝‍🧑",
            WordOpinion::Okay => "👍",
            WordOpinion::Nope => "👎",
        }
    }
}

impl Display for WordOpinion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            WordOpinion::Yes => "Yes",
            WordOpinion::Jokingly => "Jokingly",
            WordOpinion::OnlyClose => "Only if we're close",
            WordOpinion::Okay => "Okay",
            WordOpinion::Nope => "Nope",
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NowPlayingInfo {
    pub state: PlaybackState,
    pub title: String,
    pub track_id: Option<String>,
    pub album: String,
    pub album_artwork: Option<String>,
    pub artist: String,
    pub artist_artwork: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackState {
    Playing,
    Recent,
}

impl PlaybackState {
    pub fn playing(&self) -> bool {
        self == &Self::Playing
    }
}
