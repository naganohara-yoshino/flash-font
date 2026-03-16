#[cfg(target_os = "windows")]
pub(crate) mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::WindowsFontLoader as NativeFontLoader;

#[cfg(not(target_os = "windows"))]
pub(crate) mod unsupported;
#[cfg(not(target_os = "windows"))]
pub(crate) use unsupported::UnsupportedFontLoader as NativeFontLoader;
