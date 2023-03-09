use axum::{extract::{Path, Extension}, response::IntoResponse};
use maud::PreEscaped;
use reqwest::StatusCode;

use crate::{apis::mediawiki::{MediawikiClient, LoggedIn}, templates::{ExtraTemplate, HtmlTemplate}};

pub async fn page(
    Path(title): Path<String>,
    Extension(mediawiki_client): Extension<MediawikiClient<LoggedIn>>,
) -> Result<impl IntoResponse, StatusCode> {
    let (title, PreEscaped(content)) = mediawiki_client.get_page(title)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get page: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(HtmlTemplate::new(
        format!("/extras/{title}"),
        ExtraTemplate {
            title,
            content,
        }
    ).into_response().await)
}
