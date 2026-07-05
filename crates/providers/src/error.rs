use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Llm error: {0}")]
    Llm(#[from] vulcan_llm::error::LlmError),
}
