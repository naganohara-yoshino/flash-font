use std::fs;

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use etcetera::{AppStrategy, AppStrategyArgs, choose_app_strategy};
use flash_font_injector::FontManager;
use inquire::Text;
use serde::{Deserialize, Serialize};

use crate::cli::*;

pub mod cli;

#[derive(Deserialize, Serialize)]
struct Config {
    db_url: String,
    font_root: Utf8PathBuf,
}

fn load_fonts(config: &Config, ass_path: impl AsRef<Utf8Path>) -> anyhow::Result<()> {
    let s = ass_font::read_text_auto(ass_path.as_ref())?;

    flash_font::update_font_database(&config.font_root, &config.db_url)?;

    let to_load = ass_font::extract_fonts(&s)
        .iter()
        .filter_map(|f| flash_font::select_font_by_name(f, &config.db_url).ok())
        .filter_map(|v| v.first().cloned())
        .map(Utf8PathBuf::from)
        .collect::<Vec<_>>();

    let mut manager = FontManager::default();

    println!("Fonts to load: {:#?}", to_load);

    manager.load_all(to_load)?;

    Ok(())
}

fn init(
    font_root: Utf8PathBuf,
    strategy: impl AppStrategy,
    config_file: Utf8PathBuf,
) -> Result<()> {
    let data_dir = Utf8PathBuf::try_from(strategy.data_dir())?;

    fs::create_dir_all(config_file.parent().unwrap())?;
    fs::create_dir_all(&data_dir)?;

    let db_path = data_dir.join("fonts.db");
    let config = Config {
        db_url: db_path.into_string(),
        font_root,
    };

    let toml = toml::to_string_pretty(&config)?;
    fs::write(&config_file, toml)?;

    // 给用户一个操作成功的正反馈
    println!("✅ 初始化成功！配置文件已保存至:\n  {}", config_file);
    Ok(())
}

fn app_strategy() -> Result<impl AppStrategy> {
    Ok(choose_app_strategy(AppStrategyArgs {
        top_level_domain: "org".to_string(),
        author: "OpenACGN".to_string(),
        app_name: "Flash Font Ass".to_string(),
    })?)
}

pub fn run(cli: Cli) -> Result<()> {
    let strategy = app_strategy()?;
    let config_dir = Utf8PathBuf::try_from(strategy.config_dir())?;
    let config_file = config_dir.join("config.toml");

    match cli.command {
        Commands::Load(args) => {
            // 使用 with_context，如果找不到文件，会给出明确的补救建议
            let config_toml = fs::read_to_string(&config_file).with_context(|| {
                format!(
                    "找不到配置文件: {config_file}\n请先运行 `flash-font-ass init` 设置字体目录。"
                )
            })?;

            let config: Config = toml::from_str(&config_toml)?;
            load_fonts(&config, &args.subtitle)?;
        }
        Commands::Init => {
            // 用 ? 替代 if let Ok，这样如果用户在终端按 Ctrl+C，程序能干净地退出而不是强行继续
            let font_root = Text::new("请输入字体库根目录 (Font root):").prompt()?;
            init(font_root.into(), strategy, config_file)?;
        }
    }

    Ok(())
}
