#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::WindowsFontHandle as NativeFontHandle;

#[cfg(not(target_os = "windows"))]
mod unsupported;
#[cfg(not(target_os = "windows"))]
pub(crate) use unsupported::UnsupportedFontHandle as NativeFontHandle;
