use std::path::Path;

use crate::{FontError, FontHandle, FontResult};

/// Stub font handle for unsupported platforms.
///
/// All operations return [`FontError::UnsupportedPlatform`].
#[derive(Debug, Default)]
pub(crate) struct UnsupportedFontHandle;

impl FontHandle for UnsupportedFontHandle {
    fn inject(_path: impl AsRef<Path>) -> FontResult<Self> {
        Err(FontError::UnsupportedPlatform)
    }

    fn unload(&mut self) -> FontResult<()> {
        Err(FontError::UnsupportedPlatform)
    }
}
