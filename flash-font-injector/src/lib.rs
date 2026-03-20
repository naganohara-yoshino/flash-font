use std::collections::HashSet;

use camino::{Utf8Path, Utf8PathBuf};
use rayon::prelude::*;

use error::{FontError, FontResult};
use sys::NativeFontRegistry;

pub mod error;
mod sys;

pub(crate) trait FontRegistry {
    fn add_font(path: &Utf8Path) -> FontResult<()>;
    fn remove_font(path: &Utf8Path) -> FontResult<()>;
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
/// let mut manager = FontManager::default();
/// manager.load("path/to/font.ttf").unwrap();
///
/// assert!(manager.len() > 0);
///
/// manager.unload("path/to/font.ttf").unwrap();
/// ```
#[derive(Debug, Default)]
pub struct FontManager {
    loaded_fonts: HashSet<Utf8PathBuf>,
    config: FontManagerConfig,
}

#[derive(Debug, Clone)]
pub struct FontManagerConfig {
    pub keep_loaded_fonts: bool,
}

impl Default for FontManagerConfig {
    fn default() -> Self {
        Self {
            keep_loaded_fonts: true,
        }
    }
}

impl FontManager {
    /// Creates an empty `FontManager`.
    pub fn new(config: FontManagerConfig) -> Self {
        Self {
            loaded_fonts: HashSet::new(),
            config,
        }
    }

    /// Loads a font from the given file path into the system.
    ///
    /// The path is canonicalized before use, so relative paths are accepted.
    /// If the same font file is already loaded, this is a no-op that returns
    /// `Ok(())`.
    /// Expect full path
    pub fn load(&mut self, path: &Utf8Path) -> FontResult<()> {
        if !self.loaded_fonts.contains(path) {
            NativeFontRegistry::add_font(path)?;
            self.loaded_fonts.insert(path.to_path_buf());
        }

        Ok(())
    }

    pub fn load_all(&mut self, paths: Vec<Utf8PathBuf>) -> FontResult<()> {
        let to_load: Vec<_> = paths
            .into_iter()
            .filter(|path| !self.loaded_fonts.contains(path))
            .collect();

        let loaded: Vec<_> = to_load
            .into_par_iter()
            .filter_map(|path| {
                if NativeFontRegistry::add_font(&path).is_ok() {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        self.loaded_fonts.extend(loaded);

        Ok(())
    }

    /// Unloads a previously loaded font and removes it from the manager.
    ///
    /// The path is canonicalized before lookup. If the font is not currently
    /// loaded, this is a no-op that returns `Ok(())`.
    pub fn unload(&mut self, path: &Utf8Path) -> FontResult<()> {
        if self.loaded_fonts.remove(path) {
            NativeFontRegistry::remove_font(path)?;
        }

        Ok(())
    }

    pub fn unload_all(&mut self) -> FontResult<()> {
        let to_unload: Vec<_> = self.loaded_fonts.drain().collect();

        let errs: Vec<_> = to_unload
            .into_par_iter()
            .filter(|path| NativeFontRegistry::remove_font(path).is_err())
            .collect();

        if !errs.is_empty() {
            return Err(FontError::UnloadFailed(errs[0].clone()));
        }

        Ok(())
    }

    /// Returns an iterator over the canonical paths of all loaded fonts.
    pub fn loaded_fonts(&self) -> impl Iterator<Item = &Utf8Path> {
        self.loaded_fonts.iter().map(|p| p.as_path())
    }

    /// Returns the number of currently loaded fonts.
    pub fn len(&self) -> usize {
        self.loaded_fonts.len()
    }

    /// Returns `true` if no fonts are currently loaded.
    pub fn is_empty(&self) -> bool {
        self.loaded_fonts.is_empty()
    }
}

impl Drop for FontManager {
    fn drop(&mut self) {
        if !self.config.keep_loaded_fonts {
            let _ = self.unload_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager() {
        let font_path = Utf8Path::new("../fonts/方正少儿_GBK.ttf");

        let mut manager = FontManager::new(FontManagerConfig {
            keep_loaded_fonts: false,
        });

        manager.load(font_path).unwrap();
        assert!(manager.loaded_fonts.contains(font_path));
        assert_eq!(manager.len(), 1);

        // Loading the same font again should be a no-op.
        manager.load(font_path).unwrap();
        assert_eq!(manager.len(), 1);

        manager.unload(font_path).unwrap();
        assert!(!manager.loaded_fonts.contains(font_path));
        assert!(manager.is_empty());
    }
}
