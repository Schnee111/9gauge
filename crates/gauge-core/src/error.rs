//! Unified error type for the headless core. Fail-open philosophy:
//! telemetry ingestion errors degrade state, they never panic the daemon.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("http client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("sse stream error: {0}")]
    Sse(String),

    #[error("auth error: {0}")]
    Auth(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),
}
