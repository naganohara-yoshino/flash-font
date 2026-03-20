use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database query error: {0}")]
    DbQueryError(#[from] diesel::result::Error),
    #[error("Database connection error: {0}")]
    DbConnectionError(#[from] diesel::ConnectionError),
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
}

pub type AppResult<T> = Result<T, AppError>;
