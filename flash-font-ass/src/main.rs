use anyhow::Result;
use clap::Parser;
use flash_font_ass::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();

    flash_font_ass::run(cli)?;

    Ok(())
}
