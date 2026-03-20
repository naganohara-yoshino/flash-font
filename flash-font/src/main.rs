use std::{sync::mpsc, thread};

use anyhow::Error;
use camino::Utf8Path;
use diesel::prelude::*;
use flash_font::{
    db::{self},
    open_for_mmap, parser, schema,
};
use rayon::prelude::*;

pub fn main() -> Result<(), Error> {
    let font_root = Utf8Path::new(r"G:\Data\fonts\");
    let mut conn = db::initialize_db_connection("fonts.db")?;

    conn.transaction::<_, Error, _>(|tx| {
        let new_paths = flash_font::gather_and_clean_font_paths(tx, font_root)?;
        println!("✨ Found {} new fonts to parse.", new_paths.len());

        if new_paths.is_empty() {
            return Ok(());
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

        Ok(())
    })?;

    println!("✅ Database fully updated!");
    Ok(())
}
