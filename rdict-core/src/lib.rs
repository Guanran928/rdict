#![forbid(unsafe_code)]

pub mod model;
pub mod parse;
pub mod rdict;

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Input text is empty")]
    EmptyInput,

    #[error("Invalid UTF-8 database path: {0}")]
    InvalidDatabasePath(PathBuf),

    #[error("No translation results")]
    NoTranslationResults,

    #[error("Failed to parse response: {0}")]
    Parse(String),

    // Third Party
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
