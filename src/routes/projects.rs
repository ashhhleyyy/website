use std::collections::HashMap;

use axum::{extract::Path, response::IntoResponse};
use regex::Regex;
use reqwest::StatusCode;
use rust_embed::RustEmbed;

use crate::{
    markdown,
    templates::{HtmlTemplate, ProjectTemplate, ProjectsTemplate},
};

#[derive(RustEmbed)]
#[folder = "projects/"]
pub struct ProjectsAssets;

pub struct Project {
    year: String,
    slug: String,
    title: String,
    description: String,
    rendered: String,
}

impl Project {
    fn url(&self) -> String {
        format!(
            "/projects/{year}/{slug}",
            year = self.year,
            slug = self.slug,
        )
    }
}

fn load_project(filename: &str) -> Option<Project> {
    lazy_static::lazy_static! {
        static ref NAME_REGEX: Regex = Regex::new(r"([0-9]{4})-([a-z\-]+)\.md$").unwrap();
    }
    if let Some(captures) = NAME_REGEX.captures(filename) {
        let (year, slug) = (
            captures.get(1).unwrap().as_str().to_string(),
            captures.get(2).unwrap().as_str().to_string(),
        );
        if let Some(asset) = ProjectsAssets::get(filename) {
            let (metadata, html) =
                markdown::render_markdown(std::str::from_utf8(&asset.data).unwrap());
            Some(Project {
                year,
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

pub async fn project(
    Path((year, slug)): Path<(String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(post) = load_project(&format!("{}-{}.md", year, slug)) {
        Ok(HtmlTemplate::new(
            format!("/projects/{}/{}", year, slug),
            ProjectTemplate {
                title: post.title.clone(),
                description: post.description,
                content: post.rendered,
            },
        )
        .into_response()
        .await)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn index() -> impl IntoResponse {
    let mut projects_by_year = HashMap::new();
    for path in ProjectsAssets::iter() {
        if let Some(project) = load_project(&path) {
            let url = project.url();
            projects_by_year
                .entry(project.year)
                .or_insert_with(Vec::new)
                .push((project.title, url));
        }
    }

    let mut projects_by_year = projects_by_year.into_iter().collect::<Vec<_>>();

    projects_by_year.sort_by_cached_key(|(year, _)| year.clone());

    HtmlTemplate::new("/projects/", ProjectsTemplate { projects_by_year })
        .into_response()
        .await
}
