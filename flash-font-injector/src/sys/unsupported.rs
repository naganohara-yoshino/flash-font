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
}
