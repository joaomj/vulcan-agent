use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_formatting() {
        let err = CoreError::Validation("bad input".into());
        assert_eq!(err.to_string(), "Validation error: bad input");

        let err = CoreError::NotFound("session".into());
        assert_eq!(err.to_string(), "Not found: session");

        let err = CoreError::Conflict("duplicate".into());
        assert_eq!(err.to_string(), "Conflict: duplicate");

        let err = CoreError::Internal("oops".into());
        assert_eq!(err.to_string(), "Internal error: oops");
    }
}
