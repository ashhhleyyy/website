mod assets;
mod blog;
mod projects;

use std::path::Path;

use axum::{
    extract::Extension, handler::Handler, http::Uri, response::Redirect, routing::{get, any_service}, Router,
};
use reqwest::StatusCode;
use tower_http::{trace::TraceLayer, services::ServeDir};

use crate::{
    apis::{CachingFetcher, NowPlayingInfo, PronounsPageProfile},
    templates::{
        AboutTemplate, ErrorTemplate, HtmlTemplate, LinksTemplate, MusicTemplate, WordsTemplate,
    },
};

use self::assets::{background, image_script};

macro_rules! simple_template {
    ($name:ident, $path:expr, $template:ident) => {
        async fn $name() -> HtmlTemplate<$template> {
            HtmlTemplate($path.into(), $template)
        }
    };
}

simple_template!(index, "/", AboutTemplate);
simple_template!(links, "/me", LinksTemplate);

async fn about() -> Redirect {
    Redirect::permanent(Uri::from_static("/"))
}

async fn words(
    Extension(fetcher): Extension<CachingFetcher<PronounsPageProfile>>,
) -> HtmlTemplate<WordsTemplate> {
    let profile = fetcher.get().await;

    HtmlTemplate(
        "/about/words".into(),
        WordsTemplate {
            card: profile.profiles.en,
        },
    )
}

async fn music(
    Extension(fetcher): Extension<CachingFetcher<NowPlayingInfo>>,
) -> HtmlTemplate<MusicTemplate> {
    let playing = fetcher.get().await;
    HtmlTemplate("/about/music".into(), MusicTemplate { playing })
}

async fn handle_404() -> (StatusCode, HtmlTemplate<ErrorTemplate>) {
    // TODO: Get the correct path here, so the right link is highlighted anyway
    (StatusCode::NOT_FOUND, HtmlTemplate(
        "/404".into(),
        ErrorTemplate {
            error_code: 404,
            error_message: "Page not found".to_string(),
        },
    ))
}

pub fn build_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/about", get(about))
        .route("/about/words", get(words))
        .route("/about/music", get(music))
        .route("/blog/", get(blog::index))
        .route("/blog/:post", get(blog::post))
        .route("/projects/", get(projects::index))
        .route("/projects/:year/:project", get(projects::project))
        .route("/me", get(links))
        .route("/assets-gen/background.svg", get(background))
        .route("/assets-gen/image.js", get(image_script))
        // .route("/assets/*path", get(get_asset))
        .route("/api/oembed", get(assets::oembed))
        .nest(
            "/assets",
            any_service(ServeDir::new(&Path::new("assets-gen"))).handle_error(
                |err: std::io::Error| async move {
                    tracing::error!("unhandled error in static file server: {}", err);
                    (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
                },
            ),
        )
        .layer(TraceLayer::new_for_http())
        .fallback(handle_404.into_service())
}
