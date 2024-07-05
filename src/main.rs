#[macro_use]
extern crate tracing;

mod apis;
mod assets;
mod error;
mod markdown;
mod routes;
mod templates;

#[cfg(debug_assertions)]
use std::path::Path;

use apis::{
    mediawiki::MediawikiClient, CachingFetcher, NowPlayingInfo, PronounsPageProfile,
    NOWPLAYING_URL, PRONOUNS_PAGE_URL,
};
use axum::{extract::Extension, response::IntoResponse, ServiceExt};
#[cfg(debug_assertions)]
use axum::routing::any_service;
#[cfg(debug_assertions)]
use reqwest::StatusCode;
#[cfg(debug_assertions)]
use tower_http::services::ServeDir;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

macro_rules! fetch_env {
    ($name:expr) => {
        ::std::env::var($name).expect(concat!("environment variable `", $name, "` must be set!"))
    };
}

#[tokio::main]
async fn main() -> error::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pronouns_page_client =
        CachingFetcher::<PronounsPageProfile>::new(PRONOUNS_PAGE_URL.to_string()).await?;
    let nowplaying_client =
        CachingFetcher::<NowPlayingInfo>::new(NOWPLAYING_URL.to_string()).await?;

    //let mediawiki_client = MediawikiClient::new(
    //    "wiki.ashhhleyyy.dev".to_owned(),
    //    fetch_env!("MW_USERNAME"),
    //    fetch_env!("MW_PASSWORD"),
    //);

    //tracing::info!("Logging into mediawiki instance...");
    //let mediawiki_client = mediawiki_client
    //    .log_in(
    //        fetch_env!("MW_TITLE_ALLOWLIST")
    //            .split(',')
    //            .map(|s| s.to_owned())
    //            .collect::<Vec<_>>(),
    //    )
    //    .await?;

    let app = routes::build_router()
        .layer(Extension(pronouns_page_client))
        .layer(Extension(nowplaying_client));
    //.layer(Extension(mediawiki_client));

    #[cfg(debug_assertions)]
    let app = app.nest_service(
        "/assets",  
        ServeDir::new(Path::new("assets-gen"))
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
    