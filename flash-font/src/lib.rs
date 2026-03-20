use std::{collections::HashSet, fs::File, sync::mpsc, thread};

use anyhow::{Error, Result};
use camino::Utf8Path;
use diesel::prelude::*;
use rayon::prelude::*;

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
    // 分批删除，防止数据量太大撑爆 SQLite IN(...) 的 32766 参数上限
    for chunk in to_delete.chunks(10000) {
        diesel::delete(schema::font_files::table.filter(schema::font_files::path.eq_any(chunk)))
            .execute(tx)?;
    }

    // 级联清理孤儿 family name 记录
    diesel::sql_query(
        "DELETE FROM font_family_names WHERE file_id NOT IN (SELECT id FROM font_files)",
    )
    .execute(tx)?;

    // 5. 将真正需要新增解析的路径列表返回
    Ok(to_add)
}

pub fn open_for_mmap(path: &str) -> std::io::Result<File> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::OpenOptionsExt;

        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_SEQUENTIAL_SCAN;

        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_SEQUENTIAL_SCAN)
            .open(path)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let f = File::open(path)?;
        #[cfg(target_os = "linux")]
        unsafe {
            use std::os::unix::io::AsRawFd;
            libc::posix_fadvise(f.as_raw_fd(), 0, 0, libc::POSIX_FADV_WILLNEED);
        }
        Ok(f)
    }
}

pub fn update_font_database(font_root: &Utf8Path, db_url: &str) -> Result<usize, Error> {
    let mut conn = db::initialize_db_connection(db_url)?;

    conn.transaction::<_, Error, _>(|tx| {
        let new_paths = gather_and_clean_font_paths(tx, font_root)?;
        let new_paths_len = new_paths.len();

        if new_paths.is_empty() {
            return Ok(0);
        }

        let (sender, receiver) = mpsc::sync_channel(10000);

        thread::scope(|s| -> Result<(), Error> {
            s.spawn(|| {
                new_paths.into_par_iter().for_each_with(sender, |ch, path| {
                    if let Ok(data_file) = open_for_mmap(&path)
                        && let Ok(data) = unsafe { memmap2::Mmap::map(&data_file) }
                    {
                        let families = parser::get_font_family_names(&data);
                        // 将解析结果发送给主线程
                        let _ = ch.send((path, families.into_iter().collect::<Vec<_>>()));
                    }
                });
            });

            for (path, families) in receiver {
                // 获取生成的主键 ID
                let file_id: i32 = diesel::insert_into(schema::font_files::table)
                    .values(db::FontFile { path })
                    .returning(schema::font_files::id)
                    .get_result(tx)?;

                // 将提取出的名字写入
                for name in families {
                    diesel::insert_into(schema::font_family_names::table)
                        .values(db::FontFamilyName { file_id, name })
                        .execute(tx)?;
                }
            }

            Ok(())
        })?;

        Ok(new_paths_len)
    })
}

pub fn select_font_by_name(name: &str, db_url: &str) -> Result<Vec<String>, Error> {
    let mut conn = db::initialize_db_connection(db_url)?;
    let fonts: Vec<String> = schema::font_files::table
        .inner_join(
            schema::font_family_names::table
                .on(schema::font_files::id.eq(schema::font_family_names::file_id)),
        )
        .filter(schema::font_family_names::name.eq(name))
        .select(schema::font_files::path)
        .load(&mut conn)?;
    Ok(fonts)
}
