use anyhow::Result;
use camino::Utf8Path;
use diesel::prelude::*;

pub mod db;
pub mod parser;
pub mod scanner;
pub mod schema;

pub fn gather_and_clean_font_paths(
    tx: &mut diesel::SqliteConnection,
    font_root: &Utf8Path,
) -> Result<Vec<String>> {
    println!("🔍 Scanning disk paths to temporary table...");

    // 1. 创建临时表
    diesel::sql_query("DROP TABLE IF EXISTS current_disk_paths").execute(tx)?;
    diesel::sql_query(
        "CREATE TEMPORARY TABLE current_disk_paths (
                path TEXT PRIMARY KEY
            )",
    )
    .execute(tx)?;

    let mut path_chunk = Vec::with_capacity(2000);

    for font_path_buf in scanner::scan_font_directory(font_root) {
        path_chunk.push(font_path_buf.into_string());

        // 满 2000 条执行一次批量插入临时表
        if path_chunk.len() >= 2000 {
            let inserts: Vec<db::TempFontPath> = path_chunk
                .iter()
                .map(|p| db::TempFontPath { path: p })
                .collect();

            diesel::insert_into(schema::current_disk_paths::table)
                .values(&inserts)
                .execute(tx)?;
            path_chunk.clear();
        }
    }

    // 插入扫描剩余的尾部数据
    if !path_chunk.is_empty() {
        let inserts: Vec<db::TempFontPath> = path_chunk
            .iter()
            .map(|p| db::TempFontPath { path: p })
            .collect();
        diesel::insert_into(schema::current_disk_paths::table)
            .values(&inserts)
            .execute(tx)?;
    }

    println!("⚖️ Diffing changes via SQLite...");

    // 3. 利用 SQLite 极速处理差异比对
    // a. 删除已经在硬盘上消失的记录
    let deleted_count = diesel::sql_query(
        "DELETE FROM font_files WHERE path NOT IN (SELECT path FROM current_disk_paths)",
    )
    .execute(tx)?;

    // b. 清理孤儿名字 (级联清理)
    diesel::sql_query(
        "DELETE FROM font_family_names WHERE file_id NOT IN (SELECT id FROM font_files)",
    )
    .execute(tx)?;

    println!("🗑️  Deleted {} missing font records.", deleted_count);

    // c. 查询出需要新增的路径
    let new_paths: Vec<String> = diesel::sql_query(
        "SELECT path FROM current_disk_paths WHERE path NOT IN (SELECT path FROM font_files)",
    )
    .load::<db::PathResult>(tx)?
    .into_iter()
    .map(|p| p.path)
    .collect();

    Ok(new_paths)
}
