use std::collections::HashMap;

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};
use lol_html::{element, html_content::ContentType, rewrite_str, Settings};
use maud::PreEscaped;
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use crate::{
    apis::{
        fedi::{self, AccountData, PostData},
        NowPlayingInfo, PronounsPageCard,
    },
    assets::ASSET_INDEX,
    routes::blog::BlogPost,
};

macro_rules! simple_template {
    ($filename:expr, $name:ident) => {
        #[derive(Template)]
        #[template(path = $filename)]
        pub struct $name;
    };
}

simple_template!("index.html", IndexTemplate);
simple_template!("about.html", AboutTemplate);
simple_template!("links.html", LinksTemplate);
simple_template!("attribution.html", AttributionTemplate);

#[derive(Template)]
#[template(path = "words.html")]
pub struct WordsTemplate {
    pub card: PronounsPageCard,
}

#[derive(Template)]
#[template(path = "music.html")]
pub struct MusicTemplate {
    pub playing: NowPlayingInfo,
}

#[derive(Template)]
#[template(path = "blog-index.html")]
pub struct BlogIndexTemplate {
    pub posts: Vec<BlogPost>,
}

#[derive(Template)]
#[template(path = "blog-post.html")]
pub struct BlogPostTemplate {
    pub title: String,
    pub date: String,
    pub description: String,
    pub content: String,
}

#[derive(Template)]
#[template(path = "projects.html")]
pub struct ProjectsTemplate {
    pub projects_by_year: Vec<(String, Vec<(String, String)>)>,
}

#[derive(Template)]
#[template(path = "project.html")]
pub struct ProjectTemplate {
    pub title: String,
    pub description: String,
    pub content: String,
}

#[derive(Template)]
#[template(path = "extra.html")]
pub struct ExtraTemplate {
    pub title: String,
    pub content: String,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub error_code: u16,
    pub error_message: String,
}

pub struct HtmlTemplate<T> {
    pub path: String,
    pub template: T,
}

impl<T: Template> HtmlTemplate<T> {
    pub fn new<S: ToString>(path: S, template: T) -> Self {
        Self {
            path: path.to_string(),
            template,
        }
    }

    pub async fn into_response(self) -> axum::response::Response {
        match self.template.render() {
            Ok(html) => Html(rewrite_html(&self.path, &html).await).into_response(),
            Err(e) => {
                error!("Failed to render template: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to render template",
                )
                    .into_response()
            }
        }
    }
}

macro_rules! attr_rewrite {
    ($tag:literal, $attr:literal) => {
        element!(concat!($tag, "[", $attr, "]"), |el| {
            let attr = el
                .get_attribute($attr)
                .expect(concat!($attr, " was required"));

            el.set_attribute($attr, crate::assets::ASSET_INDEX.get(&attr))?;

            Ok(())
        })
    };

    ($attr:literal) => {
        attr_rewrite!("", $attr)
    };
}

async fn load_post(server: &str, id: &str) -> PostData {
    let post = fedi::POST_FETCHER
        .get_post(server.to_owned(), id.to_owned())
        .await;
    match post {
        Ok(post) => post,
        Err(e) => {
            tracing::warn!(server, id, ?e, "failed to fetch post");
            PostData {
                url: "https://oopsie.ashhhleyyy.dev/".to_owned(),
                content: "Failed to load toot!".to_owned(),
                timestamps: fedi::Timestamps::Created {
                    created_at: OffsetDateTime::UNIX_EPOCH,
                },
                account: AccountData {
                    avatar_static:
                        "https://cdn.ashhhleyyy.dev/file/ashhhleyyy-assets/images/pfp.png"
                            .to_owned(),
                    avatar: "https://cdn.ashhhleyyy.dev/file/ashhhleyyy-assets/images/pfp.png"
                        .to_owned(),
                    display_name: "Ashley".to_owned(),
                    fqn: "ash@ashhhleyyy.dev".to_owned(),
                    url: "https://ashhhleyyy.dev".to_owned(),
                },
                media_attachments: vec![],
            }
        }
    }
}

// TODO: Refactor into a tower layer(?) to remove the requirement for passing the path directly
pub(crate) async fn rewrite_html(path: &str, html: &str) -> String {
    let now = OffsetDateTime::now_utc();

    // First pass to locate fedi posts and footnotes
    let mut posts = vec![];
    let mut footnotes: Vec<Footnote> = vec![];

    struct Footnote {
        id: String,
        content: String,
        reference_count: usize,
        target_id: usize,
    }

    fn is_url(s: &str) -> bool {
        reqwest::Url::parse(s).is_ok()
    }

    let mut next_footnote_id = 0;

    let html = rewrite_str(
        html,
        Settings {
            element_content_handlers: vec![
                // This is applied in the first pass so that images get rewritten in the second one.
                element!("dialogue", |el| {
                    if let (Some(character), Some(mood)) = (el.get_attribute("character"), el.get_attribute("mood")) {
                        el.set_tag_name("blockquote")?;
                        el.set_attribute("class", "dialogue")?;
                        el.remove_attribute("character");
                        el.remove_attribute("mood");
                        let image = maud::html! {
                            // TODO: improve alt text when i rework asset handling
                            img width=(72) height=(72) src=(format!("/assets/images/characters/{character}/{mood}.png")) alt=(format!("{character}, {mood}"));
                        };
                        el.prepend("<span><b>Leah</b>:", ContentType::Html);
                        el.prepend(&image.0, ContentType::Html);
                        el.append("</span>", ContentType::Html);
                    } else {
                        el.replace("<blockquote>Oops! Invalid dialogue elemment.</blockquote>", ContentType::Html);
                    }
                    Ok(())
                }),
                element!("fedi-post", |el| {
                    if let (Some(server), Some(id)) =
                        (el.get_attribute("data-server"), el.get_attribute("data-id"))
                    {
                        posts.push((server, id));
                    } else {
                        tracing::warn!(
                            "invalid fedi-post element: missing data-server or data-id attribute!"
                        );
                        let post = PostData {
                            url: "https://oopsie.ashhhleyyy.dev/".to_owned(),
                            content: "Invalid fedi-post element!".to_owned(),
                            timestamps: fedi::Timestamps::Created {
                                created_at: OffsetDateTime::UNIX_EPOCH,
                            },
                            account: AccountData {
                                avatar_static:
                                    "https://cdn.ashhhleyyy.dev/file/ashhhleyyy-assets/images/pfp.png"
                                        .to_owned(),
                                avatar:
                                    "https://cdn.ashhhleyyy.dev/file/ashhhleyyy-assets/images/pfp.png"
                                        .to_owned(),
                                display_name: "Ashley".to_owned(),
                                fqn: "ash@ashhhleyyy.dev".to_owned(),
                                url: "https://ashhhleyyy.dev".to_owned(),
                            },
                            media_attachments: vec![],
                        };
                        el.replace(&post.as_html().0, ContentType::Html);
                    };
                    Ok(())
                }),
                element!("fn", |el| {
                    match (el.get_attribute("id"), el.get_attribute("content")) {
                        (Some(id), Some(content)) => {
                            if footnotes.iter().any(|f| f.id == id) {
                                el.replace("[duplicate footnote ID!]", ContentType::Text);
                            } else {
                                let content = if is_url(&content) {
                                    maud::html! {
                                        a href=(content) { (content) }
                                    }.0
                                } else {
                                    content
                                };
                                footnotes.push(Footnote {
                                    id,
                                    content,
                                    reference_count: 1,
                                    target_id: footnotes.len() + 1,
                                });
                                el.set_attribute("ref_no", "1")?;
                                el.set_attribute("target_id", &footnotes.len().to_string())?;
                            }
                        },
                        (None, Some(content)) => {
                            let id = format!("f~{next_footnote_id}");
                            next_footnote_id += 1;
                            let content = if is_url(&content) {
                                maud::html! {
                                    a href=(content) { (content) }
                                }.0
                            } else {
                                content
                            };
                            footnotes.push(Footnote {
                                id,
                                content,
                                reference_count: 1,
                                target_id: footnotes.len() + 1,
                            });
                            el.set_attribute("ref_no", "1")?;
                            el.set_attribute("target_id", &footnotes.len().to_string())?;
                        }
                        (Some(id), None) => {
                            if let Some(footnote) = footnotes.iter_mut().find(|f| f.id == id) {
                                footnote.reference_count += 1;
                                el.set_attribute("ref_no", &format!("{}", footnote.reference_count))?;
                                el.set_attribute("target_id", &footnote.target_id.to_string())?;
                            } else {
                                el.replace("[unknown footnote id]", ContentType::Text);
                            }
                        }
                        _ => {
                            el.replace("[footnote missing content]", ContentType::Text)
                        },
                    };
                    Ok(())
                }),
            ],
            ..Default::default()
        },
    )
    .unwrap();

    let posts = {
        let mut resolved_posts = HashMap::new();
        for (server, id) in posts {
            let key = (server, id);

            #[allow(clippy::map_entry)] // clippy's suggestion creates compiler errors
            if !resolved_posts.contains_key(&key) {
                let post = load_post(&key.0, &key.1).await;
                resolved_posts.insert(key, post);
            }
        }
        resolved_posts
    };

    rewrite_str(
        &html,
        Settings {
            element_content_handlers: vec![
                element!("copyright-year", |el| {
                    el.replace(&format!("{}", now.year()), ContentType::Text);
                    Ok(())
                }),
                element!("page-generated", |el| {
                    el.replace(
                        &now.format(&Rfc2822).expect("failed to format"),
                        ContentType::Text,
                    );
                    Ok(())
                }),
                element!("fedi-post", |el| {
                    el.replace(&posts.get(&(el.get_attribute("data-server").unwrap(), el.get_attribute("data-id").unwrap())).unwrap().as_html().0, ContentType::Html);
                    Ok(())
                }),
                element!("fn", |el| {
                    let target_id = el.get_attribute("target_id").unwrap();
                    let ref_no = el.get_attribute("ref_no").unwrap();
                    let id = format!("~fn{target_id}~{ref_no}");
                    let html = maud::html! {
                        a class="inline-note" href=(format!("#~fn{target_id}")) id=(id) {
                            sup {
                                (target_id)
                            }
                        }
                    };
                    el.replace(&html.0, ContentType::Html);
                    Ok(())
                }),
                element!("footnotes", |el| {
                    let footnotes = maud::html! {
                        ol {
                            @for footnote in &footnotes {
                                li id=(format!("~fn{}", footnote.target_id)) {
                                    @for ref_no in 0..(footnote.reference_count) {
                                        a href=(format!("#~fn{}~{}", footnote.target_id, ref_no + 1)) {
                                            (PreEscaped("&#8593;"))
                                        }
                                        " "
                                    }
                                    (PreEscaped(footnote.content.clone()))
                                }
                            }
                        }
                    };
                    el.replace(&footnotes.0, ContentType::Html);
                    Ok(())
                }),
                element!(".nav-link", |el| {
                    if let Some(href) = el.get_attribute("href") {
                        let matches = if href == "/" {
                            path == "/"
                        } else {
                            let prefix = if href.ends_with('/') {
                                href
                            } else {
                                format!("{href}/")
                            };
                            path.starts_with(&prefix)
                        };
                        if matches {
                            el.set_attribute("class", "nav-link active")?;
                        }
                        el.set_attribute("aria-current", if matches { "true" } else { "false" })?;
                    }
                    Ok(())
                }),
                element!("img[src]", |el| {
                    if el.get_attribute("data-no-rewrite").is_some() {
                        el.remove_attribute("data-no-rewrite");
                        return Ok(());
                    }
                    let src = el.get_attribute("src").expect("src required");
                    if let Some(paths) = ASSET_INDEX.get_all(&src) {
                        let html = maud::html! {
                            picture {
                                @for path in paths {
                                    @if path.ends_with(".png") {
                                        img src=[Some(path)] alt=[el.get_attribute("alt")] width=[el.get_attribute("width")] height=[el.get_attribute("height")] class=[el.get_attribute("class")];
                                    } @else {
                                        source srcset=[Some(path)] type=[mime_guess::from_path(path).first_raw()];
                                    }
                                }
                            }
                        };
                        el.replace(&html.0, ContentType::Html);
                    }
                    Ok(())
                }),
                element!(".mw-editsection", |el| {
                    el.remove();
                    Ok(())
                }),
                attr_rewrite!("src"),
                attr_rewrite!("srcset"),
                attr_rewrite!("href"),
                attr_rewrite!("meta", "content"),
                #[cfg(debug_assertions)]
                element!("head", |el| {
                    let stylesheet = concat!("<style>", include_str!("devel.css"), "</style>");
                    el.append(stylesheet, ContentType::Html);
                    let stylesheet = concat!("<style>", include_str!("devel-light.css"), "</style>");
                    el.append(stylesheet, ContentType::Html);
                    Ok(())
                }),
            ],
            ..Default::default()
        },
    )
    .unwrap()
}
