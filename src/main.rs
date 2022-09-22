#[macro_use]
extern crate tracing;

mod apis;
mod assets;
mod error;
mod markdown;
mod routes;
mod templates;

use std::path::Path;

use apis::{
    CachingFetcher, NowPlayingInfo, PronounsPageProfile, NOWPLAYING_URL, PRONOUNS_PAGE_URL,
};
use axum::{extract::Extension, routing::any_service};
use reqwest::StatusCode;
use tower_http::services::ServeDir;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

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

    let app = routes::build_router()
        .layer(Extension(pronouns_page_client))
        .layer(Extension(nowplaying_client));

    #[cfg(debug_assertions)]
    let app = app.nest(
        "/assets",
        any_service(ServeDir::new(&Path::new("assets-gen"))).handle_error(
            |err: std::io::Error| async move {
                tracing::error!("unhandled error in static file server: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
            },
        ),
    );

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
