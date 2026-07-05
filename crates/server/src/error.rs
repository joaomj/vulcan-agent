use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("Storage error: {0}")]
    Storage(#[from] vulcan_storage::error::StorageError),

    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}
