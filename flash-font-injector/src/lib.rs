use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{FontError, FontResult};
use sys::NativeFontHandle;
mod sys;

pub mod error;

/// A platform-specific handle to a temporarily loaded font resource.
///
/// This trait is intentionally **not** public — downstream users interact with
/// [`FontManager`] instead. Each platform provides its own implementation
/// (e.g. `WindowsFontHandle` on Windows).
pub(crate) trait FontHandle
where
    Self: Sized,
{
    /// Creates a new handle from a font file path without loading the font.
    fn new(path: impl AsRef<Path>) -> FontResult<Self>;

    /// Loads the font into the operating system.
    ///
    /// Implementations **must** guard against double-loading: calling `load()`
    /// on an already-loaded handle should be a no-op that returns `Ok(())`.
    fn load(&mut self) -> FontResult<()>;

    /// Unloads the font from the operating system.
    ///
    /// Implementations **must** guard against double-unloading: calling
    /// `unload()` on an already-unloaded handle should be a no-op that returns
    /// `Ok(())`.
    fn unload(&mut self) -> FontResult<()>;
}

/// Manages the lifecycle of temporarily loaded system fonts.
///
/// Fonts are keyed by their canonical (absolute) path, ensuring each physical
/// font file is loaded at most once. When the manager is dropped, all loaded
/// fonts are automatically unloaded.
///
/// # Examples
///
/// ```no_run
/// use flash_font_injector::FontManager;
///
/// let mut manager = FontManager::new();
/// manager.load("path/to/font.ttf").unwrap();
///
/// assert!(manager.is_loaded("path/to/font.ttf"));
///
/// manager.unload("path/to/font.ttf").unwrap();
/// ```
#[derive(Debug, Default)]
pub struct FontManager {
    fonts: HashMap<PathBuf, NativeFontHandle>,
}

impl FontManager {
    /// Creates an empty `FontManager`.
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    /// Loads a font from the given file path into the system.
    ///
    /// The path is canonicalized before use, so relative paths are accepted.
    /// If the same font file is already loaded, this is a no-op that returns
    /// `Ok(())`.
    pub fn load(&mut self, path: impl AsRef<Path>) -> FontResult<()> {
        let canonical = path.as_ref().canonicalize()?;

        if self.fonts.contains_key(&canonical) {
            return Ok(());
        }

        let mut handle = NativeFontHandle::new(&canonical)?;
        handle.load()?;
        self.fonts.insert(canonical, handle);
        Ok(())
    }

    /// Unloads a previously loaded font and removes it from the manager.
    ///
    /// The path is canonicalized before lookup. If the font is not currently
    /// loaded, this is a no-op that returns `Ok(())`.
    pub fn unload(&mut self, path: impl AsRef<Path>) -> FontResult<()> {
        let canonical = path.as_ref().canonicalize()?;

        if let Some(mut handle) = self.fonts.remove(&canonical) {
            handle.unload()?;
        }
        Ok(())
    }

    /// Unloads all currently loaded fonts.
    ///
    /// Errors during individual unloads are printed to `stderr` but do not
    /// prevent the remaining fonts from being unloaded.
    pub fn unload_all(&mut self) {
        for (path, mut handle) in self.fonts.drain() {
            if let Err(e) = handle.unload() {
                eprintln!("warning: failed to unload font {}: {e}", path.display());
            }
        }
    }

    /// Returns `true` if the font at the given path is currently loaded.
    ///
    /// Returns `false` if the path cannot be canonicalized.
    pub fn is_loaded(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref()
            .canonicalize()
            .map(|p| self.fonts.contains_key(&p))
            .unwrap_or(false)
    }

    /// Returns an iterator over the canonical paths of all loaded fonts.
    pub fn loaded_fonts(&self) -> impl Iterator<Item = &Path> {
        self.fonts.keys().map(|p| p.as_path())
    }

    /// Returns the number of currently loaded fonts.
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    /// Returns `true` if no fonts are currently loaded.
    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }
}

impl Drop for FontManager {
    fn drop(&mut self) {
        self.unload_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager() {
        let font_path = "../fonts/方正少儿_GBK.ttf";

        let mut manager = FontManager::new();
        manager.load(font_path).unwrap();
        assert!(manager.is_loaded(font_path));
        assert_eq!(manager.len(), 1);

        // Loading the same font again should be a no-op.
        manager.load(font_path).unwrap();
        assert_eq!(manager.len(), 1);

        manager.unload(font_path).unwrap();
        assert!(!manager.is_loaded(font_path));
        assert!(manager.is_empty());
    }
}
