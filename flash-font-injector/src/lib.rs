use std::io;
use std::path::{self, Path, PathBuf};

use thiserror::Error;
use widestring::WideCString;
use widestring::error::ContainsNul;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{AddFontResourceW, RemoveFontResourceW};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_FONTCHANGE,
};
use windows::core::PCWSTR;

/// Represents errors that can occur during font loading or unloading operations.
#[derive(Debug, Error)]
pub enum FontLoadingError {
    /// Indicates that the font failed to load into the system.
    #[error("failed to load font: `{0}`")]
    LoadFailed(PathBuf),
    /// Indicates that the font failed to unload from the system.
    #[error("failed to unload font: `{0}`")]
    UnloadFailed(PathBuf),
    /// Indicates that the provided font path could not be converted to a valid format (e.g., contained null bytes).
    #[error("failed to convert path: `{0}`")]
    PathConversionFailed(#[from] ContainsNul<u16>),
    /// Indicates that the provided font path could not be converted to an absolute path.
    #[error("failed to get absolute path: `{0}`")]
    AbsolutePathFailed(#[from] io::Error),
}

/// Result type for font operations.
pub type FontLoadingResult<T> = Result<T, FontLoadingError>;

/// A guard that strictly manages the lifecycle of a globally loaded font.
///
/// Ensures the font is loaded into the Windows system and automatically unloads it
/// when the guard goes out of scope, unless explicitly unloaded beforehand.
#[derive(Debug, Clone, Default)]
pub struct GlobalFontGuard {
    path: PathBuf,
    path_w: Vec<u16>,
}

impl GlobalFontGuard {
    /// Loads the specified font into the system globally.
    ///
    /// The loaded font is available system-wide until this guard is explicitly unloaded
    /// or dropped. This function automatically broadcasts a font change message to
    /// ensure windows applications are notified of the new font.
    ///
    /// # Errors
    ///
    /// Returns a [`FontLoadingError`] if the font fails to load or if the path is invalid.
    pub fn load(path: impl AsRef<Path>) -> FontLoadingResult<Self> {
        let path_buf: PathBuf = path::absolute(path.as_ref())?;

        // Format the path as a wide string (UTF-16) for Windows API calls.
        let path_w = WideCString::from_os_str(path_buf.as_os_str())?.into_vec_with_nul();

        unsafe {
            // 1. Load the font resource into the system.
            let added = AddFontResourceW(PCWSTR(path_w.as_ptr()));
            if added == 0 {
                return Err(FontLoadingError::LoadFailed(path_buf));
            }

            // 2. Notify other applications that a font has been added.
            Self::broadcast_font_change();
        }

        Ok(Self {
            path: path_buf,
            path_w,
        })
    }

    /// Explicitly unloads the font before the guard goes out of scope.
    ///
    /// This allows for manual control over when the font is removed, rather than
    /// waiting for the guard to be dropped automatically.
    ///
    /// # Errors
    ///
    /// Returns a [`FontLoadingError`] if the system fails to unload the font.
    pub fn unload(self) -> FontLoadingResult<()> {
        let result = self.do_unload();

        // Explicitly forget the guard to prevent `Drop` from unloading it a second time.
        std::mem::forget(self);
        result
    }

    /// Internal method to remove the font from the system.
    ///
    /// Called by both the explicit `unload` method and the `Drop` implementation to
    /// ensure the font is always properly cleaned up.
    fn do_unload(&self) -> FontLoadingResult<()> {
        unsafe {
            let removed = RemoveFontResourceW(PCWSTR(self.path_w.as_ptr()));
            if removed.0 == 0 {
                return Err(FontLoadingError::UnloadFailed(self.path.clone()));
            }

            Self::broadcast_font_change();
        }
        Ok(())
    }

    /// Broadcasts a font change message `WM_FONTCHANGE`.
    ///
    /// This notifies all top-level windows in the system of the font update.
    fn broadcast_font_change() {
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_FONTCHANGE,
                WPARAM(0),
                LPARAM(0),
                SMTO_ABORTIFHUNG,
                1000,
                None,
            );
        }
    }
}

/// Ensures the font is unloaded when the guard goes out of scope,
/// providing an automatic cleanup mechanism if not unloaded explicitly.
impl Drop for GlobalFontGuard {
    fn drop(&mut self) {
        let _ = self.do_unload();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_load_and_unload() {
        let font_path: &Path = Path::new("../fonts/方正少儿_GBK.ttf");

        let font_guard = GlobalFontGuard::load(font_path).unwrap();

        font_guard.unload().unwrap();
    }
}
