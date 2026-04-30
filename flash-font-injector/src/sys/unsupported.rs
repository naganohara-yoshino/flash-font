use camino::Utf8Path;

use crate::{FontError, FontRegistry, FontResult};

#[derive(Debug, Default)]
pub(crate) struct UnsupportedFontRegistry;

impl FontRegistry for UnsupportedFontRegistry {
    fn add_font(_path: &Utf8Path) -> FontResult<()> {
        Err(FontError::UnsupportedPlatform)
    }
    fn remove_font(_path: &Utf8Path) -> FontResult<()> {
        Err(FontError::UnsupportedPlatform)
    }
    fn is_font_available(_family_name: &str) -> FontResult<bool> {
        Err(FontError::UnsupportedPlatform)
    }
}
