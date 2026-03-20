use camino::Utf8Path;

use widestring::WideCString;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{AddFontResourceW, RemoveFontResourceW};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_FONTCHANGE,
};
use windows::core::PCWSTR;

use crate::{FontError, FontRegistry, FontResult};

/// A Windows-specific font handle that uses the GDI `AddFontResourceW` /
/// `RemoveFontResourceW` APIs to temporarily load a font into the system.
#[derive(Debug, Default)]
pub(crate) struct WinRegistry;

impl WinRegistry {
    /// Broadcasts `WM_FONTCHANGE` to all top-level windows so they can pick up
    /// the font change. Prints a warning to `stderr` if the broadcast times
    /// out (e.g. because a window is hung).
    #[allow(dead_code)]
    pub(crate) fn broadcast_font_change() -> Result<(), ()> {
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
            return Err(());
        }

        Ok(())
    }
}

impl FontRegistry for WinRegistry {
    fn add_font(path: &Utf8Path) -> FontResult<()> {
        let path_w = WideCString::from_str(path.as_str())
            .map_err(|e| FontError::MalformedPath(e.to_string()))?
            .into_vec_with_nul();

        let added = unsafe { AddFontResourceW(PCWSTR(path_w.as_ptr())) };

        match added {
            0 => Err(FontError::LoadFailed(path.to_path_buf())),
            _ => Ok(()),
        }
    }

    fn remove_font(path: &Utf8Path) -> FontResult<()> {
        let path_w = WideCString::from_str(path.as_str())
            .map_err(|e| FontError::MalformedPath(e.to_string()))?
            .into_vec_with_nul();

        let removed = unsafe { RemoveFontResourceW(PCWSTR(path_w.as_ptr())) };

        match removed.0 {
            0 => Err(FontError::UnloadFailed(path.to_path_buf())),
            _ => Ok(()),
        }
    }
}
