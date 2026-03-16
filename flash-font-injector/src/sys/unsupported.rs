use std::path::Path;
use crate::{FontLoader, FontLoadingError, FontLoadingResult};

pub struct UnsupportedFontLoader;

impl FontLoader for UnsupportedFontLoader {
    type Handle = ();

    fn load(_path: &Path) -> FontLoadingResult<Self::Handle> {
        Err(FontLoadingError::UnsupportedPlatform)
    }

    fn unload(_handle: &mut Self::Handle) -> FontLoadingResult<()> {
        Ok(())
    }
}
