use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
};
use maud::PreEscaped;
use reqwest::StatusCode;

use crate::{
    apis::mediawiki::{LoggedIn, MediawikiClient},
    templates::{ExtraTemplate, HtmlTemplate},
};

pub async fn page(
    Path(title): Path<String>,
    Extension(mediawiki_client): Extension<MediawikiClient<LoggedIn>>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some((title, PreEscaped(content))) =
        mediawiki_client.get_page(title).await.map_err(|e| {
            tracing::error!("Failed to get page: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    {
        Ok(
            HtmlTemplate::new(format!("/extras/{title}"), ExtraTemplate { title, content })
                .into_response()
                .await,
        )
    } else {
        Ok(super::handle_404().await)
    }
}
