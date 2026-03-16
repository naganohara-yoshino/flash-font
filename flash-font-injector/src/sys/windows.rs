use std::path::{self, Path, PathBuf};
use widestring::WideCString;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{AddFontResourceW, RemoveFontResourceW};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_FONTCHANGE,
};
use windows::core::PCWSTR;

use crate::{FontLoader, FontLoadingError, FontLoadingResult};

#[derive(Debug, Clone, Default)]
pub struct WindowsFontHandle {
    pub path: PathBuf,
    pub path_w: Vec<u16>,
}

pub struct WindowsFontLoader;

impl WindowsFontLoader {
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

impl FontLoader for WindowsFontLoader {
    type Handle = WindowsFontHandle;

    fn load(path: &Path) -> FontLoadingResult<Self::Handle> {
        let path_buf = path::absolute(path)?;

        let path_w = WideCString::from_os_str(path_buf.as_os_str())
            .map_err(|e| FontLoadingError::InvalidPath(e.to_string()))?
            .into_vec_with_nul();

        unsafe {
            let added = AddFontResourceW(PCWSTR(path_w.as_ptr()));
            if added == 0 {
                return Err(FontLoadingError::LoadFailed(path_buf));
            }
            Self::broadcast_font_change();
        }

        Ok(WindowsFontHandle {
            path: path_buf,
            path_w,
        })
    }

    fn unload(handle: &mut Self::Handle) -> FontLoadingResult<()> {
        unsafe {
            let removed = RemoveFontResourceW(PCWSTR(handle.path_w.as_ptr()));
            if removed.0 == 0 {
                return Err(FontLoadingError::UnloadFailed(handle.path.clone()));
            }
            Self::broadcast_font_change();
        }
        Ok(())
    }
}
