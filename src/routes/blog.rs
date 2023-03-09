use regex::Regex;
use rust_embed::RustEmbed;

use axum::{extract::Path, response::IntoResponse};

use crate::{
    markdown,
    templates::{BlogIndexTemplate, BlogPostTemplate, HtmlTemplate},
};

#[derive(RustEmbed)]
#[folder = "blog/"]
pub struct BlogAssets;

pub struct BlogPost {
    year: String,
    month: String,
    day: String,
    slug: String,
    title: String,
    description: String,
    rendered: String,
}

impl BlogPost {
    fn url(&self) -> String {
        format!(
            "/blog/{year}-{month}-{day}-{slug}",
            year = self.year,
            month = self.month,
            day = self.day,
            slug = self.slug,
        )
    }

    fn date(&self) -> String {
        format!(
            "{year}-{month}-{day}",
            year = self.year,
            month = self.month,
            day = self.day
        )
    }
}

fn load_post(filename: &str) -> Option<BlogPost> {
    lazy_static::lazy_static! {
        static ref NAME_REGEX: Regex = Regex::new(r"([0-9]{4})-([0-9]{2})-([0-9]{2})-([a-z0-9\-]+)\.md$").unwrap();
    }
    if let Some(captures) = NAME_REGEX.captures(filename) {
        let (year, month, day) = (
            captures.get(1).unwrap().as_str().to_string(),
            captures.get(2).unwrap().as_str().to_string(),
            captures.get(3).unwrap().as_str().to_string(),
        );
        let slug = captures.get(4).unwrap().as_str().to_string();
        if let Some(asset) = BlogAssets::get(filename) {
            let (metadata, html) =
                markdown::render_markdown(std::str::from_utf8(&asset.data).unwrap());
            Some(BlogPost {
                year,
                month,
                day,
                slug,
                title: metadata.title,
                description: metadata.description,
                rendered: html,
            })
        } else {
            None
        }
    } else {
        None
    }
}

pub async fn index() -> impl IntoResponse {
    let mut entries = vec![];
    for path in BlogAssets::iter() {
        if let Some(post) = load_post(&path) {
            let url = post.url();
            let date = post.date();
            entries.push((date, post.title, url));
        }
    }
    entries.sort_by(|a, b| b.0.cmp(&a.0));
    HtmlTemplate::new("/blog/", BlogIndexTemplate { posts: entries })
        .into_response()
        .await
}

pub async fn post(Path(path): Path<String>) -> impl IntoResponse {
    if let Some(post) = load_post(&format!("{}.md", path)) {
        HtmlTemplate::new(
            format!("/blog/{}", path),
            BlogPostTemplate {
                title: post.title.clone(),
                date: post.date(),
                description: post.description,
                content: post.rendered,
            },
        )
        .into_response()
        .await
    } else {
        super::handle_404().await
    }
}
