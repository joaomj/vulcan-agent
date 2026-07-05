use thiserror::Error;

#[derive(Error, Debug)]
pub enum GuardrailError {
    #[error("Denied: {0}")]
    Denied(String),

    #[error("Approval required: {0}")]
    ApprovalRequired(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
