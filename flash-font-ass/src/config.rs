use std::fs;

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

/// Configuration data for the `flash-font-ass` CLI.
#[derive(Deserialize, Serialize)]
pub(crate) struct Config {
    pub(crate) db_url: String,
    pub(crate) font_root: Utf8PathBuf,
    pub(crate) should_show_notifications: bool,
}

impl Config {
    pub(crate) fn load(path: &Utf8Path) -> Result<Self> {
        let config_toml = fs::read_to_string(path).with_context(|| {
            format!("Can't find config file: {path}\nPlease run `flash-font-ass init` first.")
        })?;
        let config: Self = toml::from_str(&config_toml)
            .with_context(|| format!("Failed to parse config file: {path}"))?;

        Ok(config)
    }

    pub(crate) fn save(&self, path: &Utf8Path) -> Result<()> {
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("config file must have a parent directory"))?;

        fs::create_dir_all(parent)?;
        fs::write(path, toml::to_string_pretty(self)?)?;

        Ok(())
    }
}
