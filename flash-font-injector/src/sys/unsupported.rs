use camino::Utf8Path;

use crate::{FontError, FontRegistry, FontResult};

#[derive(Debug, Default)]
pub(crate) struct UnsupportedFontRegistry;

impl FontRegistry for UnsupportedFontHandle {
    fn add_font(path: &Utf8Path) -> FontResult<()> {
        Err(FontError::UnsupportedPlatform)
    }
    fn remove_font(path: &Utf8Path) -> FontResult<()> {
        Err(FontError::UnsupportedPlatform)
    }
}
