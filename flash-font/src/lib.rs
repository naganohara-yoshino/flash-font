//! A library for managing a font database, including scanning, parsing, and searching fonts.
//!
//! Provides utilities to synchronize font files on disk with a SQLite database
//! and search for font files by their family names.

use std::{collections::HashSet, fs::File, io, sync::mpsc, thread};

use camino::Utf8Path;
use diesel::prelude::*;
use rayon::prelude::*;

use crate::error::{AppError, AppResult};

mod db;
pub mod error;
pub mod parser;
pub mod scanner;
mod schema;

/// Synchronizes the font files in the database with the files currently on disk.
///
/// This function identifies files that have been removed from the disk and deletes their
/// corresponding records from the database. It then returns a list of new file paths
/// that need to be parsed and added.
fn gather_and_clean_font_paths(
    tx: &mut diesel::SqliteConnection,
    font_root: &Utf8Path,
) -> AppResult<Vec<String>> {
    // 1. Get all current paths in the database
    let db_paths: HashSet<String> = schema::font_files::table
        .select(schema::font_files::path)
        .load(tx)?
        .into_iter()
        .collect();

    let disk_paths: HashSet<String> = scanner::scan_font_directory(font_root)
        .map(|p| p.into_string())
        .collect();

    // 3. Compute difference in memory (extremely fast)
    // In DB but not on disk -> needs deletion
    let to_delete: Vec<String> = db_paths.difference(&disk_paths).cloned().collect();
    // On disk but not in DB -> needs addition
    let to_add: Vec<String> = disk_paths.difference(&db_paths).cloned().collect();

    // 4. Clean up invalid records in the database
    // Delete in chunks to avoid hitting the SQLite parameter limit of 32766
    for chunk in to_delete.chunks(10000) {
        diesel::delete(schema::font_files::table.filter(schema::font_files::path.eq_any(chunk)))
            .execute(tx)?;
    }

    // Cascade clean orphaned font family name records
    diesel::sql_query(
        "DELETE FROM font_family_names WHERE file_id NOT IN (SELECT id FROM font_files)",
    )
    .execute(tx)?;

    // 5. Return the list of paths that truly need to be parsed and added
    Ok(to_add)
}

/// Opens a file with performance-optimized flags for memory mapping.
///
/// On Windows, it uses `FILE_FLAG_SEQUENTIAL_SCAN` to hint to the OS that
/// the file will be read sequentially.
fn open_for_mmap(path: &str) -> io::Result<File> {
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

/// Updates the font database by scanning the provided root directory.
///
/// This function synchronizes the database with the disk, removes stale entries,
/// parses new font files in parallel to extract family names, and inserts them
/// into the database.
///
/// Returns the number of new font files added.
pub fn update_font_database(font_root: &Utf8Path, db_url: &str) -> AppResult<usize> {
    let mut conn = db::initialize_db_connection(db_url)?;

    conn.transaction::<_, AppError, _>(|tx| {
        let new_paths = gather_and_clean_font_paths(tx, font_root)?;
        let new_paths_len = new_paths.len();

        if new_paths.is_empty() {
            return Ok(0);
        }

        let (sender, receiver) = mpsc::sync_channel(10000);

        thread::scope(|s| -> AppResult<()> {
            s.spawn(|| {
                new_paths.into_par_iter().for_each_with(sender, |ch, path| {
                    if let Ok(data_file) = open_for_mmap(&path)
                        && let Ok(data) = unsafe { memmap2::Mmap::map(&data_file) }
                    {
                        let families = parser::get_font_family_names(&data);
                        // Send parsing results back to the main thread
                        let _ = ch.send((path, families.into_iter().collect::<Vec<_>>()));
                    }
                });
            });

            for (path, families) in receiver {
                // Get the generated primary key ID
                let file_id: i32 = diesel::insert_into(schema::font_files::table)
                    .values(db::FontFile { path })
                    .returning(schema::font_files::id)
                    .get_result(tx)?;

                // Write the extracted names to the database
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

/// Searches the database for font file paths matching a specific font family name.
pub fn select_font_by_name(name: &str, db_url: &str) -> AppResult<Vec<String>> {
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
