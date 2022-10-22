use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};
use lol_html::{element, html_content::ContentType, rewrite_str, Settings};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use crate::{
    apis::{NowPlayingInfo, PronounsPageCard},
    assets::ASSET_INDEX,
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
    pub posts: Vec<(String, String, String)>,
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
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub error_code: u16,
    pub error_message: String,
}

pub struct HtmlTemplate<T>(pub String, pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> axum::response::Response {
        match self.1.render() {
            Ok(html) => Html(rewrite_html(&self.0, &html)).into_response(),
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

// TODO: Refactor into a tower layer(?) to remove the requirement for passing the path directly
fn rewrite_html(path: &str, html: &str) -> String {
    let now = OffsetDateTime::now_utc();
    rewrite_str(
        html,
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
                element!(".blog-post a", |el| {
                    // Only external links
                    if matches!(el.get_attribute("href"), Some(href) if href.starts_with("https://") && !el.has_attribute("target")) {
                        el.set_attribute("target", "_blank")?;
                    }
                    Ok(())
                }),
                element!(".nav-link", |el| {
                    if let Some(href) = el.get_attribute("href") {
                        let mtchs = if href == "/" {
                            path == "/"
                        } else {
                            let prefix = if href.ends_with('/') {
                                href
                            } else {
                                format!("{}/", href)
                            };
                            path.starts_with(&prefix)
                        };
                        if mtchs {
                            el.set_attribute("class", "nav-link active")?;
                        }
                    }
                    Ok(())
                }),
                element!("img[src]", |el| {
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
                attr_rewrite!("src"),
                attr_rewrite!("href"),
                attr_rewrite!("meta", "content"),
                #[cfg(debug_assertions)]
                element!("head", |el| {
                    let stylesheet = concat!("<style>", include_str!("devel.css"), "</style>");
                    el.append(stylesheet, ContentType::Html);
                    Ok(())
                }),
            ],
            ..Default::default()
        },
    )
    .unwrap()
}
