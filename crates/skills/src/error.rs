use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
