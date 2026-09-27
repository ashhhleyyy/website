use axum::{extract::Path, response::IntoResponse};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;

use crate::{
    markdown,
    templates::{GalleriesTemplate, GalleryTemplate, HtmlTemplate},
};

#[derive(rust_embed::Embed)]
#[folder = "galleries/"]
pub struct GalleryAssets;

#[derive(Deserialize)]
pub struct Photo {
    pub filename: String,
    pub alt: String,
    #[serde(default)]
    pub caption: Option<String>,
}

#[derive(Deserialize)]
pub struct GalleryMetadata {
    pub title: String,
    #[serde(default)]
    pub spoiler: Option<String>,
    #[serde(default)]
    pub unlisted: bool,
    pub photos: Vec<Photo>,
}

impl Default for GalleryMetadata {
    fn default() -> Self {
        Self {
            title: "WARNING! An error occured while parsing the frontmatter".to_owned(),
            spoiler: Some("WARNING! An error occured while parsing the frontmatter".to_owned()),
            unlisted: false,
            photos: Vec::with_capacity(0),
        }
    }
}

pub struct Gallery {
    pub year: String,
    pub month: String,
    pub day: String,
    pub slug: String,
    pub title: String,
    pub spoiler: Option<String>,
    pub unlisted: bool,
    pub images: Vec<Photo>,
    pub rendered: String,
}

impl Gallery {
    pub fn url(&self) -> String {
        format!("/photos/{slug}", slug = self.slug())
    }

    pub fn slug(&self) -> String {
        format!(
            "{year}-{month}-{day}-{slug}",
            year = self.year,
            month = self.month,
            day = self.day,
            slug = self.slug,
        )
    }

    pub fn date(&self) -> String {
        format!(
            "{year}-{month}-{day}",
            year = self.year,
            month = self.month,
            day = self.day
        )
    }
}

fn load_gallery(filename: &str) -> Option<Gallery> {
    static NAME_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"([0-9]{4})-([0-9]{2})-([0-9]{2})-([a-z0-9\-]+)\.md$").unwrap());
    if let Some(captures) = NAME_REGEX.captures(filename) {
        let (year, month, day) = (
            captures.get(1).unwrap().as_str().to_string(),
            captures.get(2).unwrap().as_str().to_string(),
            captures.get(3).unwrap().as_str().to_string(),
        );
        let slug = captures.get(4).unwrap().as_str().to_string();
        if let Some(asset) = GalleryAssets::get(filename) {
            let (metadata, html) = markdown::render_markdown::<GalleryMetadata>(
                std::str::from_utf8(&asset.data).unwrap(),
            );
            Some(Gallery {
                year,
                month,
                day,
                slug,
                title: metadata.title,
                spoiler: metadata.spoiler,
                unlisted: metadata.unlisted,
                images: metadata.photos,
                rendered: html,
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn list_galleries() -> Vec<Gallery> {
    let mut posts = GalleryAssets::iter()
        .filter_map(|path| load_gallery(&path))
        .filter(|g| !g.unlisted)
        .collect::<Vec<_>>();
    posts.sort_by_key(|g| g.date());
    posts.reverse();
    posts
}

pub async fn gallery(Path(path): Path<String>) -> impl IntoResponse {
    if let Some(gallery) = load_gallery(&format!("{path}.md")) {
        HtmlTemplate::new(
            format!("/photos/{path}"),
            GalleryTemplate {
                slug: gallery.slug(),
                title: gallery.title.clone(),
                date: gallery.date(),
                spoiler: gallery.spoiler,
                content: gallery.rendered,
                photos: gallery.images,
            },
        )
        .into_response()
        .await
    } else {
        super::handle_404().await
    }
}

pub async fn index() -> impl IntoResponse {
    let galleries = list_galleries();
    HtmlTemplate::new("/photos/", GalleriesTemplate { galleries })
        .into_response()
        .await
}
