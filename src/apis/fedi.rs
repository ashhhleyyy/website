use std::{collections::HashMap, sync::Arc};

use time::OffsetDateTime;
use tokio::sync::Mutex;

use crate::error::Result;

use super::CachingFetcher;

lazy_static::lazy_static! {
    pub(crate) static ref POST_FETCHER: CachingPostFetcher = CachingPostFetcher::new();
}

pub struct CachingPostFetcher {
    #[allow(clippy::type_complexity)]
    fetchers: Arc<Mutex<HashMap<(String, String), CachingFetcher<PostData>>>>,
}

impl CachingPostFetcher {
    pub fn new() -> Self {
        Self {
            fetchers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_post(&self, server: String, id: String) -> Result<PostData> {
        let mut fetchers = self.fetchers.lock().await;

        let fetcher = if let Some(fetcher) = fetchers.get_mut(&(server.clone(), id.clone())) {
            fetcher
        } else {
            let fetcher =
                CachingFetcher::<PostData>::new(format!("https://{server}/api/v1/statuses/{id}"))
                    .await?;
            fetchers.insert((server.clone(), id.clone()), fetcher);
            fetchers.get_mut(&(server, id)).unwrap()
        };

        Ok(fetcher.get().await)
    }
}

#[derive(Clone, serde::Deserialize)]
pub struct PostData {
    pub content: String,
    pub account: AccountData,
    pub url: String,
    pub media_attachments: Vec<Attatchment>,
    #[serde(flatten)]
    pub timestamps: Timestamps,
}

#[derive(Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum Timestamps {
    Created {
        #[serde(with = "time::serde::rfc3339")]
        created_at: OffsetDateTime,
    },
    Edited {
        #[serde(with = "time::serde::rfc3339")]
        created_at: OffsetDateTime,
        #[serde(with = "time::serde::rfc3339")]
        edited_at: OffsetDateTime,
    },
}

#[derive(Clone, serde::Deserialize)]
pub struct AccountData {
    pub avatar_static: String,
    pub avatar: String,
    pub display_name: String,
    pub fqn: String,
    pub url: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct Attatchment {
    pub description: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub url: String,
}

fn format_odt(date: OffsetDateTime) -> String {
    let (hour, min) = (date.hour(), date.minute());
    let (year, month, date) = (date.year(), date.month() as u8, date.day());
    format!("at {hour:#02}:{min:#02} on {year:#04}-{month:#02}-{date:#02}")
}

impl PostData {
    pub fn as_html(&self) -> maud::Markup {
        maud::html! {
            .fedi-post {
                blockquote {
                    .fedi-author {
                        img.fedi-avatar width="48" height="48" src=(self.account.avatar_static);

                        a href=(self.account.url) {
                            (self.account.display_name) " (@" (self.account.display_name) ")"
                        }
                    }

                    br;

                    (maud::PreEscaped(self.content.clone()))

                    @if !self.media_attachments.is_empty() {
                        br;
                    }

                    @for attachment in &self.media_attachments {
                        @if attachment.ty == "image" {
                            img src=(attachment.url) alt=(attachment.description);
                        }
                    }

                    br; br;

                    @match self.timestamps {
                        Timestamps::Created { created_at } => {
                            "Posted " (format_odt(created_at))
                        },
                        Timestamps::Edited { created_at, edited_at } => {
                            "Posted " (format_odt(created_at)) " (Edited at " (format_odt(edited_at)) ")"
                        },
                    }
                }
            }
        }
    }
}
