mod assets;
mod blog;
mod extras;
mod projects;

use axum::{
    extract::Extension,
    handler::Handler,
    http::Uri,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use reqwest::StatusCode;
use tower_http::trace::TraceLayer;

use crate::{
    apis::{CachingFetcher, NowPlayingInfo, PronounsPageProfile},
    templates::{
        AboutTemplate, AttributionTemplate, ErrorTemplate, HtmlTemplate, LinksTemplate, MusicTemplate, WordsTemplate,
    },
};

use self::assets::{background, image_script};

macro_rules! simple_template {
    ($name:ident, $path:expr, $template:ident) => {
        async fn $name() -> impl IntoResponse {
            HtmlTemplate::new($path, $template).into_response().await
        }
    };
}

simple_template!(index, "/", AboutTemplate);
simple_template!(links, "/me", LinksTemplate);
simple_template!(attribution, "/attribution", AttributionTemplate);

async fn about() -> Redirect {
    Redirect::permanent("/")
}

async fn words(
    Extension(fetcher): Extension<CachingFetcher<PronounsPageProfile>>,
) -> impl IntoResponse {
    let profile = fetcher.get().await;

    HtmlTemplate::new(
        "/about/words",
        WordsTemplate {
            card: profile.profiles.en,
        },
    )
    .into_response()
    .await
}

async fn music(Extension(fetcher): Extension<CachingFetcher<NowPlayingInfo>>) -> impl IntoResponse {
    let playing = fetcher.get().await;
    HtmlTemplate::new("/about/music", MusicTemplate { playing })
        .into_response()
        .await
}

async fn handle_404() -> Response {
    // TODO: Get the correct path here, so the right link is highlighted anyway
    (
        StatusCode::NOT_FOUND,
        HtmlTemplate::new(
            "/404",
            ErrorTemplate {
                error_code: 404,
                error_message: "Page not found".to_string(),
            },
        )
        .into_response()
        .await,
    )
        .into_response()
}

pub fn build_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/about", get(about))
        .route("/about/words", get(words))
        .route("/about/music", get(music))
        .route("/attribution", get(attribution))
        .route("/blog/", get(blog::index))
        .route("/blog/:post", get(blog::post))
        .route("/projects/", get(projects::index))
        .route("/projects/:year/:project", get(projects::project))
        //.route("/extras/:title", get(extras::page))
        .route("/me", get(links))
        .route("/assets-gen/background.svg", get(background))
        .route("/assets-gen/image.js", get(image_script))
        .route("/api/oembed", get(assets::oembed))
        .layer(TraceLayer::new_for_http())
        .fallback(handle_404)
}
