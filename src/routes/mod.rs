mod assets;
mod blog;
mod projects;

use axum::{
    extract::Extension, handler::Handler, http::Uri, response::Redirect, routing::get, Router,
};
use tower_http::trace::TraceLayer;

use crate::{
    apis::{CachingFetcher, NowPlayingInfo, PronounsPageProfile},
    templates::{
        AboutTemplate, ErrorTemplate, HtmlTemplate, LinksTemplate, MusicTemplate, WordsTemplate,
    },
};

use self::assets::{background, get_asset, image_script};

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

async fn handle_404() -> HtmlTemplate<ErrorTemplate> {
    // TODO: Get the correct path here, so the right link is highlighted anyway
    HtmlTemplate(
        "/404".into(),
        ErrorTemplate {
            error_code: 404,
            error_message: "Page not found".to_string(),
        },
    )
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
        .route("/assets/*path", get(get_asset))
        .route("/api/oembed", get(assets::oembed))
        .layer(TraceLayer::new_for_http())
        .fallback(handle_404.into_service())
}
