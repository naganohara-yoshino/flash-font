use camino::Utf8PathBuf;
use thiserror::Error;

/// Represents errors that can occur during font operations.
#[derive(Debug, Error, Clone)]
pub enum FontError {
    /// The font failed to load into the system.
    #[error("failed to load font: `{0}`")]
    LoadFailed(Utf8PathBuf),
    /// The font failed to unload from the system.
    #[error("failed to unload font: `{0}`")]
    UnloadFailed(Utf8PathBuf),
    /// The provided font path could not be converted to a valid format.
    #[error("malformed font path: `{0}`")]
    MalformedPath(String),
    /// The current platform is not supported.
    #[error("unsupported platform")]
    UnsupportedPlatform,
}

/// Result type alias for font operations.
pub type FontResult<T> = Result<T, FontError>;
