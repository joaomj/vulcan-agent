use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecallError {
    #[error("Storage error: {0}")]
    Storage(#[from] vulcan_storage::error::StorageError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
