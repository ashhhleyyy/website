use std::{sync::Arc, time::{Instant, Duration}, fmt::Display};

use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, de::{Visitor, DeserializeOwned}};
use serde_repr::Deserialize_repr;
use tokio::sync::{Mutex, RwLock};

use crate::error::Result;

const USER_AGENT: &str = "ashhhleyyy.dev website backend (v1, https://github.com/ashhhleyyy/)";
pub const PRONOUNS_PAGE_URL: &str = "https://en.pronouns.page/api/profile/get/ashhhleyyy";
pub const NOWPLAYING_URL: &str = "https://api.ashhhleyyy.dev/playing";
const MIN_REFRESH_TIME: Duration = Duration::from_secs(5);

lazy_static::lazy_static!{
    static ref CLIENT: Client = ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build().expect("failed to build client");
}

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
        let req = CLIENT.get(url)
            .build()?;

        let res = CLIENT.execute(req).await?
            .error_for_status()?;

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
    pub names: Words,
    pub pronouns: Words,
    pub flags: Vec<String>,
    pub words: Vec<Words>,
}

#[derive(Clone)]
pub struct Words(pub Vec<(String, WordOpinion)>);

struct WordsVisitor;

impl<'de> Visitor<'de> for WordsVisitor {
    type Value = Words;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map of words")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error> where A: serde::de::MapAccess<'de>, {
        let mut words = Words(Vec::with_capacity(map.size_hint().unwrap_or(0)));
        while let Some((key, value)) = map.next_entry()? {
            words.0.push((key, value));
        }
        Ok(words)
    }
}

impl<'de> Deserialize<'de> for Words {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        deserializer.deserialize_map(WordsVisitor)
    }
}

#[derive(Clone, Copy, Deserialize_repr, Eq, PartialEq)]
#[repr(i8)]
pub enum WordOpinion {
    Yes = 1,
    Jokingly = 2,
    OnlyClose = 3,
    Okay = 0,
    Nope = -1,
}

impl WordOpinion {
    pub fn is_negative(&self) -> bool {
        match self {
            WordOpinion::Yes
                | WordOpinion::Jokingly
                | WordOpinion::OnlyClose
                | WordOpinion::Okay
                => false,
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
        return self == &Self::Playing;
    }
}
