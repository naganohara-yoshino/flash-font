#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::WinRegistry as NativeFontRegistry;

#[cfg(not(target_os = "windows"))]
mod unsupported;
#[cfg(not(target_os = "windows"))]
pub(crate) use unsupported::UnsupportedFontRegistry as NativeFontRegistry;
