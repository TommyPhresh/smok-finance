use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("DB error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Row not found: {0}")]
    NotFound(String),

    #[error("Data validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
