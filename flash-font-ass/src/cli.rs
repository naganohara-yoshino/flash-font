use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
/// A CLI tool for loading ASS subtitle fonts
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
    #[arg(short, long)]
    pub(crate) subtitle: Utf8PathBuf,
}
