use std::io;
use std::result;

use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("SoundCloud error: {0}")]
    ApiError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("HTTP error: {0}")]
    HttpHeaderError(#[from] reqwest::header::ToStrError),

    #[error("HTTP error: {0}")]
    HttpInvalidHeaderError(#[from] reqwest::header::InvalidHeaderValue),

    #[error("Invalid filter")]
    InvalidFilter(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("The track is not available for download")]
    TrackNotDownloadable,

    #[error("The track is not available for streaming")]
    TrackNotStreamable,

    #[error("URL parsing error: {0}")]
    UrlParseError(#[from] url::ParseError),
}
