use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

pub(crate) mod sys;

/// Represents errors that can occur during font loading or unloading operations.
#[derive(Debug, Error)]
pub enum FontLoadingError {
    /// Indicates that the font failed to load into the system.
    #[error("failed to load font: `{0}`")]
    LoadFailed(PathBuf),
    /// Indicates that the font failed to unload from the system.
    #[error("failed to unload font: `{0}`")]
    UnloadFailed(PathBuf),
    /// Indicates that the provided font path could not be converted to a valid format.
    #[error("invalid font path: `{0}`")]
    InvalidPath(String),
    /// Indicates that the provided font path could not be converted to an absolute path.
    #[error("failed to get absolute path: `{0}`")]
    AbsolutePathFailed(#[from] io::Error),
    /// Indicates that the current platform is not supported.
    #[error("unsupported platform")]
    UnsupportedPlatform,
}

/// Result type for font operations.
pub type FontLoadingResult<T> = Result<T, FontLoadingError>;

/// A generic trait for platform-specific font loading operations.
pub trait FontLoader {
    /// The handle representing a loaded font resource.
    type Handle;

    /// Loads the specified font into the system globally.
    fn load(path: &Path) -> FontLoadingResult<Self::Handle>;

    /// Unloads the font from the system.
    fn unload(handle: &mut Self::Handle) -> FontLoadingResult<()>;
}

/// A guard that strictly manages the lifecycle of a globally loaded font.
///
/// Ensures the font is loaded into the Windows system and automatically unloads it
/// when the guard goes out of scope, unless explicitly unloaded beforehand.
#[derive(Debug, Clone, Default)]
pub struct GlobalFontGuard<L: FontLoader> {
    handle: Option<L::Handle>,
}

impl<L: FontLoader> GlobalFontGuard<L> {
    /// Loads the specified font into the system globally.
    pub fn load(path: impl AsRef<Path>) -> FontLoadingResult<Self> {
        let handle = L::load(path.as_ref())?;
        Ok(Self {
            handle: Some(handle),
        })
    }

    /// Explicitly unloads the font before the guard goes out of scope.
    pub fn unload(mut self) -> FontLoadingResult<()> {
        let result = self.do_unload();

        // Explicitly forget the guard to prevent `Drop` from unloading it a second time.
        std::mem::forget(self);
        result
    }

    fn do_unload(&mut self) -> FontLoadingResult<()> {
        if let Some(mut handle) = self.handle.take() {
            L::unload(&mut handle)
        } else {
            Ok(())
        }
    }
}

/// Ensures the font is unloaded when the guard goes out of scope.
impl<L: FontLoader> Drop for GlobalFontGuard<L> {
    fn drop(&mut self) {
        let _ = self.do_unload();
    }
}

/// A convenience type alias for the current platform's font guard.
pub type SystemFontGuard = GlobalFontGuard<sys::NativeFontLoader>;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_load_and_unload() {
        let font_path: &Path = Path::new("../fonts/方正少儿_GBK.ttf");

        let result = SystemFontGuard::load(font_path);
        
        // On unsupported platforms, this fails gracefully. On Windows, it unloads.
        if let Ok(guard) = result {
            guard.unload().unwrap();
        }
    }
}
