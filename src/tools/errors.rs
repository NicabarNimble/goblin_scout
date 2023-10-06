// tools/errors.rs
use git2::Error as GitError;
use serde_json::Error as JsonError;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error(transparent)]
    IOError(#[from] io::Error),

    #[error("Encountered error: {0}")]
    StrError(String),

    #[error(transparent)]
    GitError(#[from] GitError),

    #[error("JSON Parsing Error: {0}")]
    JsonParsingError(#[from] JsonError),

    #[error("Failed to parse {0}. Reason: {1}")]
    DetailedJsonParsingError(String, String),
}
