use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Represents errors that can occur during font operations.
#[derive(Debug, Error)]
pub enum FontError {
    /// The font failed to load into the system.
    #[error("failed to load font: `{0}`")]
    LoadFailed(PathBuf),
    /// The font failed to unload from the system.
    #[error("failed to unload font: `{0}`")]
    UnloadFailed(PathBuf),
    /// The provided font path could not be converted to a valid format.
    #[error("invalid font path: `{0}`")]
    InvalidPath(String),
    /// The provided font path could not be resolved to an absolute path.
    #[error("failed to resolve path: `{0}`")]
    IoError(#[from] io::Error),
    /// The current platform is not supported.
    #[error("unsupported platform")]
    UnsupportedPlatform,
}

/// Result type alias for font operations.
pub type FontResult<T> = Result<T, FontError>;
