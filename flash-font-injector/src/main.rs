use std::path::{Path, PathBuf};

use thiserror::Error;
use widestring::WideCString;
use widestring::error::ContainsNul;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{AddFontResourceW, RemoveFontResourceW};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_FONTCHANGE,
};
use windows::core::PCWSTR;

#[derive(Debug, Error, Clone)]
pub enum FontLoadingError {
    #[error("加载字体失败: {0}")]
    LoadFailed(PathBuf),
    #[error("卸载字体失败: {0}")]
    UnloadFailed(PathBuf),
    #[error("路径转换失败: {0}")]
    PathConversionFailed(#[from] ContainsNul<u16>),
}

pub type FontLoadingResult<T> = Result<T, FontLoadingError>;

#[derive(Debug, Clone, Default)]
pub struct GlobalFontGuard {
    path: PathBuf,
    path_w: Vec<u16>,
}

impl GlobalFontGuard {
    pub fn load<P: AsRef<Path>>(path: P) -> FontLoadingResult<Self> {
        let path_buf = path.as_ref().to_path_buf();

        // format the path as a wide string (UTF-16) for Windows API calls
        let path_w = WideCString::from_os_str(path_buf.as_os_str())?.into_vec_with_nul();

        unsafe {
            // 1. load the font into the system
            let added = AddFontResourceW(PCWSTR(path_w.as_ptr()));
            if added == 0 {
                return Err(FontLoadingError::LoadFailed(path_buf));
            }

            // 2. notify the system of the new font
            Self::broadcast_font_change();
        }

        Ok(Self {
            path: path_buf,
            path_w,
        })
    }

    /// explictly unload the font before the guard goes out of scope. This allows for manual control over when the font is removed, rather than waiting for the guard to be dropped.
    pub fn unload(self) -> FontLoadingResult<()> {
        let result = self.do_unload();

        // 显式卸载后，阻止 Drop 再次执行以防重复卸载
        std::mem::forget(self);
        result
    }

    /// unload the font from the system. This is called by both the explicit unload method and the Drop implementation to ensure that the font is removed regardless of how the guard is used.
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

    /// Broadcast a font change message to notify the system of the font update.
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
/// Ensure that the font is unloaded when the guard goes out of scope, even if the user forgets to call unload explicitly.
impl Drop for GlobalFontGuard {
    fn drop(&mut self) {
        let _ = self.do_unload();
    }
}

fn main() -> anyhow::Result<()> {
    let font_path = r"C:\Users\Anna\Desktop\New folder\方正少儿GBK.ttf";

    let font_guard = GlobalFontGuard::load(font_path)?;

    println!("字体加载成功！按 Enter 键卸载该字体...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    println!("正在卸载字体...");
    font_guard.unload()?;
    println!("字体显式卸载成功！");

    Ok(())
}
