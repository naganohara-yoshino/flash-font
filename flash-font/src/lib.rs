use std::collections::HashSet;

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
    // 1. 获取数据库中现有的全量路径
    let db_paths: HashSet<String> = schema::font_files::table
        .select(schema::font_files::path)
        .load(tx)?
        .into_iter()
        .collect();

    let disk_paths: HashSet<String> = scanner::scan_font_directory(font_root)
        .map(|p| p.into_string())
        .collect();

    // 3. 内存直接求差集（极其快速）
    // 数据库中有，但磁盘没有 -> 需要删除
    let to_delete: Vec<String> = db_paths.difference(&disk_paths).cloned().collect();
    // 磁盘上有，但数据库没有 -> 需要新增
    let to_add: Vec<String> = disk_paths.difference(&db_paths).cloned().collect();

    // 4. 清理数据库中失效的记录
    if !to_delete.is_empty() {
        println!(
            "🗑️  Found {} missing font records. Deleting...",
            to_delete.len()
        );
        let mut deleted_count = 0;

        // 分批删除，防止数据量太大撑爆 SQLite IN(...) 的 32766 参数上限
        for chunk in to_delete.chunks(10000) {
            deleted_count += diesel::delete(
                schema::font_files::table.filter(schema::font_files::path.eq_any(chunk)),
            )
            .execute(tx)?;
        }

        println!("🗑️  Deleted {} missing font records.", deleted_count);

        // 级联清理孤儿 family name 记录
        diesel::sql_query(
            "DELETE FROM font_family_names WHERE file_id NOT IN (SELECT id FROM font_files)",
        )
        .execute(tx)?;
    }

    println!("✨ Found {} new fonts to parse.", to_add.len());

    // 5. 将真正需要新增解析的路径列表返回
    Ok(to_add)
}
