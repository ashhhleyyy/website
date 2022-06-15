use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};
use lol_html::{rewrite_str, Settings, element, html_content::ContentType};
use time::{OffsetDateTime, format_description::well_known::Rfc2822};

use crate::apis::{NowPlayingInfo, PronounsPageCard};

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

// TODO: Refactor into a tower layer(?) to remove the requirement for passing the path directly
fn rewrite_html(path: &str, html: &str) -> String {
    let now = OffsetDateTime::now_utc();
    rewrite_str(html, Settings {
        element_content_handlers: vec![
            element!("copyright-year", |el| {
                el.replace(&format!("{}", now.year()), ContentType::Text);
                Ok(())
            }),
            element!("page-generated", |el| {
                el.replace(&now.format(&Rfc2822).expect("failed to format"), ContentType::Text);
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
            })
        ],
        ..Default::default()
    }).unwrap()
}
