#[derive(Debug, thiserror::Error)]
pub enum WebsiteError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, WebsiteError>;
