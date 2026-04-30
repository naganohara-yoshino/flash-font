use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use flash_font_injector::{FontManager, is_font_available};

use crate::config::Config;

fn extract_fonts(subtitle: &Utf8Path) -> Result<Vec<String>> {
    let ass_text = ass_font::read_text_auto(subtitle)
        .with_context(|| format!("Failed to read subtitle file: {subtitle}"))?;

    Ok(ass_font::extract_fonts(&ass_text))
}

/// Represents the result of font selection after querying the database.
struct FontSelection {
    valid_paths: Vec<Utf8PathBuf>,
    missing: Vec<String>,
}

fn resolve_fonts(font_names: &[String], db_url: &str) -> Result<FontSelection> {
    let mut valid_paths = Vec::new();
    let mut missing = Vec::new();

    for font_name in font_names {
        let selected_path = flash_font::select_font_by_name(font_name, db_url)
            .with_context(|| format!("Failed to query font database for `{font_name}`"))?
            .into_iter()
            .next();

        if let Some(path) = selected_path {
            valid_paths.push(Utf8PathBuf::from(path));
        } else {
            missing.push(font_name.clone());
        }
    }

    let missing = missing
        .into_iter()
        .filter(|font_name| !matches!(is_font_available(font_name), Ok(true)))
        .collect();

    Ok(FontSelection {
        valid_paths,
        missing,
    })
}

pub(crate) struct LoadReport {
    loaded_count: usize,
    missing_fonts: Vec<String>,
}

impl LoadReport {
    pub(crate) fn message(&self) -> String {
        format!(
            "Loaded {} fonts. Missing: {}",
            self.loaded_count,
            if self.missing_fonts.is_empty() {
                "none".to_string()
            } else {
                self.missing_fonts.join(", ")
            }
        )
    }
}

pub(crate) fn load_fonts(config: &Config, subtitle: &Utf8Path) -> Result<LoadReport> {
    let font_names = extract_fonts(subtitle)?;

    flash_font::update_font_database(&config.font_root, &config.db_url)
        .with_context(|| format!("Failed to update font database from {}", config.font_root))?;

    let selection = resolve_fonts(&font_names, &config.db_url)?;

    let mut manager = FontManager::default();
    manager.load_all(selection.valid_paths)?;

    let loaded_count = manager.len();

    Ok(LoadReport {
        loaded_count,
        missing_fonts: selection.missing,
    })
}
