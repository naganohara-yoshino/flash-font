use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};

/// A CLI tool for loading ASS subtitle fonts
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Load ASS subtitle fonts
    Load(LoadArgs),
    /// Initialize configuration file
    Init,
}

#[derive(Parser, Debug)]
pub(crate) struct LoadArgs {
    #[arg(short, long, value_name = "ASS_FILE")]
    pub(crate) subtitle: Utf8PathBuf,
}
