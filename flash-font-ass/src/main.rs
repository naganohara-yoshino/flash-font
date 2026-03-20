use std::io;

use camino::{Utf8Path, Utf8PathBuf};
use flash_font_injector::{FontManager, FontManagerConfig};

pub fn main() {
    let s = ass_font::read_text_auto(
        r"E:\qb_downloads\[VCB-Studio] Sora yori mo Tooi Basho [Ma10p_1080p]\[VCB-Studio] Sora yori mo Tooi Basho [01][Ma10p_1080p][x265_flac_aac].ass",
    )
    .unwrap();

    let db_url = r"F:\Coding\fonts.db";

    flash_font::update_font_database(Utf8Path::new(r"G:\Data\fonts"), db_url).unwrap();

    let to_load = ass_font::extract_fonts(&s)
        .iter()
        .filter_map(|f| flash_font::select_font_by_name(f, db_url).ok())
        .filter_map(|v| v.first().cloned())
        .map(Utf8PathBuf::from)
        .collect::<Vec<_>>();

    let mut manager = FontManager::new(FontManagerConfig {
        keep_loaded_fonts: false,
    });

    manager.load_all(to_load).unwrap();

    println!("回车退出");
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
}
