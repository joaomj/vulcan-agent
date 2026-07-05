use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Cancelled")]
    Cancelled,

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
