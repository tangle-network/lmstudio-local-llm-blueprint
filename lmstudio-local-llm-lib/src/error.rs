use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Blueprint SDK error: {0}")]
    Sdk(#[from] blueprint_sdk::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to execute lms command: {0}")]
    LmsCommandFailed(String),

    #[error("Failed to parse lms output: {0}")]
    LmsOutputParse(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid job arguments: {0}")]
    InvalidArgs(String),
}

pub type Result<T> = std::result::Result<T, Error>;
