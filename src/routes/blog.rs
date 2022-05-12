use regex::Regex;
use rust_embed::RustEmbed;

use axum::{http::StatusCode, extract::Path};

use crate::{templates::{HtmlTemplate, BlogIndexTemplate, BlogPostTemplate}, copyright_year, generated};

#[derive(RustEmbed)]
#[folder = "blog/"]
pub struct BlogAssets;

pub struct BlogPost {
    year: String,
    month: String,
    day: String,
    slug: String,
    title: String,
    rendered: String,
}

impl BlogPost {
    fn url(&self) -> String {
        format!("/blog/{year}-{month}-{day}-{slug}", year = self.year, month = self.month, day = self.day, slug = self.slug)
    }

    fn date(&self) -> String {
        format!("{year}-{month}-{day}", year = self.year, month = self.month, day = self.day)
    }
}

fn load_post(filename: &str) -> Option<BlogPost> {
    let name_regex = Regex::new(r"([0-9]{4})-([0-9]{2})-([0-9]{2})-([a-z\-]+)\.md$").unwrap();
    // Yes I know you're not *supposed* to parse html with regex
    let title_regex = Regex::new(r"<h1>(.*)</h1>").unwrap();
    if let Some(captures) = name_regex.captures(filename) {
        let (year, month, day) = (
            captures.get(1).unwrap().as_str().to_string(),
            captures.get(2).unwrap().as_str().to_string(),
            captures.get(3).unwrap().as_str().to_string(),
        );
        let slug = captures.get(4).unwrap().as_str().to_string();
        if let Some(asset) = BlogAssets::get(filename) {
            let html = comrak::markdown_to_html(&String::from_utf8(asset.data.to_vec()).unwrap(), &Default::default());
            let title = title_regex.captures(&html).unwrap().get(1).unwrap().as_str().to_string();
            Some(BlogPost {
                year,
                month,
                day,
                slug,
                title,
                rendered: html,
            })
        } else {
            None
        }
    } else {
        None
    }
}

pub async fn index() -> HtmlTemplate<BlogIndexTemplate> {
    let mut entries = vec![];
    for path in BlogAssets::iter() {
        if let Some(post) = load_post(&path) {
            let url = post.url();
            let date = post.date();
            entries.push((date, post.title, url));
        }
    }
    return HtmlTemplate(BlogIndexTemplate {
        copyright_year: copyright_year!(),
        generated: generated!(),
        posts: entries,
    })
}

pub async fn post(Path(path): Path<String>,) -> Result<HtmlTemplate<BlogPostTemplate>, StatusCode> {
    if let Some(post) = load_post(&format!("{}.md", path)) {
        Ok(HtmlTemplate(BlogPostTemplate {
            copyright_year: copyright_year!(),
            generated: generated!(),
            title: post.title.clone(),
            date: post.date().to_string(),
            content: post.rendered,
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
