use atom_syndication::{
    Content, EntryBuilder, FixedDateTime, Generator, LinkBuilder, Person, Text,
};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header::CONTENT_TYPE;
use rss::ItemBuilder;
use rust_embed::RustEmbed;

use axum::{
    extract::Path,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use time::{format_description::well_known::Rfc2822, Date, Month, OffsetDateTime, Time};

use crate::{
    markdown,
    templates::{BlogIndexTemplate, BlogPostTemplate, HtmlTemplate},
};

#[derive(RustEmbed)]
#[folder = "blog/"]
pub struct BlogAssets;

pub struct BlogPost {
    pub year: String,
    pub month: String,
    pub day: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub rendered: String,
}

impl BlogPost {
    pub fn url(&self) -> String {
        format!(
            "/blog/{year}-{month}-{day}-{slug}",
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

fn load_post(filename: &str) -> Option<BlogPost> {
    static NAME_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"([0-9]{4})-([0-9]{2})-([0-9]{2})-([a-z0-9\-]+)\.md$").unwrap());
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

struct FeedPost {
    post: BlogPost,
    html: String,
}

impl FeedPost {
    async fn from(post: BlogPost) -> Self {
        let html = crate::templates::rewrite_html(&post.url(), &post.rendered).await;
        Self { post, html }
    }
}

fn list_posts() -> Vec<BlogPost> {
    let mut posts = BlogAssets::iter()
        .filter_map(|path| load_post(&path))
        .collect::<Vec<_>>();
    posts.sort_by_key(|p| p.date());
    posts.reverse();
    posts
}

async fn posts_feed() -> Vec<FeedPost> {
    let posts = list_posts();
    let mut feed = Vec::with_capacity(posts.len());
    for post in posts {
        feed.push(FeedPost::from(post).await);
    }
    feed
}

pub async fn index() -> impl IntoResponse {
    let posts = list_posts();
    HtmlTemplate::new("/blog/", BlogIndexTemplate { posts })
        .into_response()
        .await
}

fn month_from_index(index: u8) -> Month {
    match index {
        1 => Month::January,
        2 => Month::February,
        3 => Month::March,
        4 => Month::April,
        5 => Month::May,
        6 => Month::June,
        7 => Month::July,
        8 => Month::August,
        9 => Month::September,
        10 => Month::October,
        11 => Month::November,
        12 => Month::December,
        _ => panic!("invalid month"),
    }
}

pub async fn rss() -> impl IntoResponse {
    let posts = posts_feed().await;
    let mut builder = rss::ChannelBuilder::default();

    builder
        .title("ash's blog")
        .link("https://ashhhleyyy.dev")
        .description("random words i write for people to read")
        .generator(Some(
            "ashhhleyyy.dev/1.0 (+https://git.ashhhleyyy.dev/mirror/website)".to_owned(),
        ))
        .last_build_date(crate::SERVER_START_TIME.format(&Rfc2822).unwrap());
    for post in posts {
        builder.item(
            ItemBuilder::default()
                .title(post.post.title.clone())
                .link(format!("https://ashhhleyyy.dev{}", post.post.url()))
                .description(post.post.description.clone())
                .guid(
                    rss::GuidBuilder::default()
                        .value(post.post.url())
                        .permalink(false)
                        .build(),
                )
                .pub_date(
                    OffsetDateTime::new_utc(
                        Date::from_calendar_date(
                            post.post.year.parse().unwrap(),
                            month_from_index(post.post.month.parse().unwrap()),
                            post.post.day.parse().unwrap(),
                        )
                        .unwrap(),
                        Time::from_hms(0, 0, 0).unwrap(),
                    )
                    .format(&Rfc2822)
                    .unwrap(),
                )
                .content(post.html)
                .build(),
        );
    }
    let feed = builder.build();
    let headers = {
        let mut headers = HeaderMap::new();
        headers.append(
            CONTENT_TYPE,
            HeaderValue::from_static("application/rss+xml"),
        );
        headers
    };
    (headers, feed.to_string())
}

pub async fn atom() -> impl IntoResponse {
    let posts = posts_feed().await;
    let mut builder = atom_syndication::FeedBuilder::default();

    builder
        .title(Text::plain("ash's blog"))
        .id("https://ashhhleyyy.dev")
        // hate
        .updated(
            FixedDateTime::parse_from_rfc2822(&crate::SERVER_START_TIME.format(&Rfc2822).unwrap())
                .unwrap(),
        )
        .author(Person {
            name: "ashhhleyyy".to_owned(),
            uri: Some("https://ashhhleyyy.dev".to_owned()),
            ..Default::default()
        })
        .generator(Generator {
            value: "ashhhleyyy.dev".to_owned(),
            uri: Some("https://git.ashhhleyyy.dev/mirror/website".to_owned()),
            version: Some("1.0".to_owned()),
        })
        .icon("https://cdn.ashhhleyyy.dev/files/ashhhleyyy-assets/images/pfp.png".to_owned())
        .link(
            LinkBuilder::default()
                .href("https://ashhhleyyy.dev/blog.atom")
                .rel("self")
                .build(),
        )
        .link(
            LinkBuilder::default()
                .href("https://ashhhleyyy.dev/")
                .build(),
        )
        .logo("https://cdn.ashhhleyyy.dev/files/ashhhleyyy-assets/images/pfp.png".to_owned())
        .subtitle(Text::plain("random words i write for people to read"))
        .base("https://ashhhleyyy.dev".to_owned());

    for post in posts {
        let posted = FixedDateTime::parse_from_rfc3339(&format!(
            "{}-{}-{}T00:00:00Z",
            post.post.year, post.post.month, post.post.day
        ))
        .unwrap();
        let url = format!("https://ashhhleyyy.dev{}", post.post.url());
        builder.entry(
            EntryBuilder::default()
                .title(Text::plain(&post.post.title))
                .id(post.post.url())
                .updated(posted)
                .author(Person {
                    name: "ashhhleyyy".to_owned(),
                    uri: Some("https://ashhhleyyy.dev".to_owned()),
                    ..Default::default()
                })
                .link(LinkBuilder::default().href(&url).build())
                .published(posted)
                .summary(Text::plain(post.post.description))
                .content(Content {
                    base: Some(url.clone()),
                    src: Some(url.clone()),
                    value: Some(post.html),
                    content_type: Some("html".to_string()),
                    ..Default::default()
                })
                .build(),
        );
    }

    let feed = builder.build();
    let headers = {
        let mut headers = HeaderMap::new();
        headers.append(
            CONTENT_TYPE,
            HeaderValue::from_static("application/atom+xml"),
        );
        headers
    };
    (headers, feed.to_string())
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
