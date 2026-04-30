use std::fs;

use anyhow::{Context, Result, bail};
use camino::Utf8PathBuf;
use etcetera::{AppStrategy, AppStrategyArgs, choose_app_strategy};
use flash_font::update_font_database;
use inquire::Text;
use notify_rust::{Notification, Timeout};

use crate::cli::*;
use crate::config::*;
use crate::font::LoadReport;

pub mod cli;
mod config;
mod font;

/// The name of the SQLite database file for fonts.
const DB_FILE: &str = "fonts.db";

struct AppPaths {
    config_file: Utf8PathBuf,
    data_dir: Utf8PathBuf,
}

impl AppPaths {
    fn determine() -> Result<Self> {
        let strategy = choose_app_strategy(AppStrategyArgs {
            top_level_domain: "org".to_string(),
            author: "OpenACGN".to_string(),
            app_name: "Flash Font Ass".to_string(),
        })?;

        let config_dir = Utf8PathBuf::try_from(strategy.config_dir())?;
        let data_dir = Utf8PathBuf::try_from(strategy.data_dir())?;

        Ok(Self {
            config_file: config_dir.join("config.toml"),
            data_dir,
        })
    }
}

fn normalize_font_root(font_root: Utf8PathBuf) -> Result<Utf8PathBuf> {
    if font_root.as_str().is_empty() {
        bail!("font root cannot be empty");
    }

    if !font_root.is_absolute() {
        bail!("font root must be an absolute path: {font_root}");
    }

    let font_root = font_root
        .canonicalize_utf8()
        .with_context(|| format!("font root must exist: {font_root}"))?;

    let metadata = font_root
        .metadata()
        .with_context(|| format!("failed to inspect font root: {font_root}"))?;

    if !metadata.is_dir() {
        bail!("font root must be a directory: {font_root}");
    }

    Ok(font_root)
}

fn prompt_for_font_root() -> Result<Utf8PathBuf> {
    let input = Text::new("Please enter the full path to the font root directory:").prompt()?;

    let trimmed_input = input.trim().trim_matches('"');

    let valid_font_root = normalize_font_root(trimmed_input.into())?;

    Ok(valid_font_root)
}

fn show_notification(report: &LoadReport) {
    if let Err(error) = Notification::new()
        .summary("ASS Fonts")
        .body(&report.message())
        .timeout(Timeout::Never)
        .show()
    {
        eprintln!("Warning: failed to show notification: {error}");
    }
}

/// Main entry point for the CLI operations.
pub fn run(cli: Cli) -> Result<()> {
    let paths = AppPaths::determine()?;

    match cli.command {
        Commands::Init => {
            let valid_font_root = prompt_for_font_root()?;

            fs::create_dir_all(&paths.data_dir)?;

            let config = Config {
                db_url: paths.data_dir.join(DB_FILE).into(),
                font_root: valid_font_root,
            };
            config.save(&paths.config_file)?;

            println!("Success! Config file saved to:\n  {}", &paths.config_file);

            update_font_database(&config.font_root, &config.db_url)?;

            println!("Font database initialized at:\n  {}", &config.db_url);
        }
        Commands::Load(LoadArgs { ref subtitle, .. }) => {
            let config = Config::load(&paths.config_file)?;
            let report = font::load_fonts(&config, subtitle)?;

            println!("{}", report.message());
            show_notification(&report);
        }
    }

    Ok(())
}
