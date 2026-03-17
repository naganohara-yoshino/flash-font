use std::path::{Path, PathBuf};

use widestring::WideCString;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{AddFontResourceW, RemoveFontResourceW};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_FONTCHANGE,
};
use windows::core::PCWSTR;

use crate::{FontError, FontHandle, FontResult};

/// A Windows-specific font handle that uses the GDI `AddFontResourceW` /
/// `RemoveFontResourceW` APIs to temporarily load a font into the system.
#[derive(Debug, Default)]
pub(crate) struct WindowsFontHandle {
    path_buf: PathBuf,
    path_w: Vec<u16>,
}

impl WindowsFontHandle {
    /// Broadcasts `WM_FONTCHANGE` to all top-level windows so they can pick up
    /// the font change. Prints a warning to `stderr` if the broadcast times
    /// out (e.g. because a window is hung).
    fn broadcast_font_change() {
        let result = unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_FONTCHANGE,
                WPARAM(0),
                LPARAM(0),
                SMTO_ABORTIFHUNG,
                1000,
                None,
            )
        };

        if result.0 == 0 {
            eprintln!("warning: WM_FONTCHANGE broadcast timed out or failed");
        }
    }
}

impl FontHandle for WindowsFontHandle {
    fn new(path: impl AsRef<Path>) -> FontResult<Self> {
        let path_buf = path.as_ref().canonicalize()?;

        let path_w = WideCString::from_os_str(path_buf.as_os_str())
            .map_err(|e| FontError::InvalidPath(e.to_string()))?
            .into_vec_with_nul();

        Ok(Self { path_buf, path_w })
    }

    fn load(&mut self) -> FontResult<()> {
        unsafe {
            let added = AddFontResourceW(PCWSTR(self.path_w.as_ptr()));
            if added == 0 {
                return Err(FontError::LoadFailed(self.path_buf.clone()));
            }
            Self::broadcast_font_change();
        }

        Ok(())
    }

    fn unload(&mut self) -> FontResult<()> {
        unsafe {
            let removed = RemoveFontResourceW(PCWSTR(self.path_w.as_ptr()));
            if removed.0 == 0 {
                return Err(FontError::UnloadFailed(self.path_buf.clone()));
            }
            Self::broadcast_font_change();
        }

        Ok(())
    }
}
