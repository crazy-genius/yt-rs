mod constants;
mod clients;
mod models;

pub use clients::*;
pub use models::*;

use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum YoutrackError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Uri(#[from] url::ParseError),

    #[error(transparent)]
    Decode(#[from] serde_json::Error),

    #[error("Unexpected HTTP status {status}: {body}")]
    UnexpectedStatus { status: StatusCode, body: String },

    #[error("Api error {0}")]
    ApiError(models::ApiError),
}

pub type Result<T> = std::result::Result<T, YoutrackError>;
