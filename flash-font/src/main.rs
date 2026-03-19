use anyhow::Error;
use camino::Utf8Path;
use diesel::prelude::*;
use flash_font::{db, open_for_mmap, parser, schema};
use rayon::prelude::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let font_root = Utf8Path::new(r"G:\Data\fonts\");
    let mut conn = db::initialize_db_connection("fonts.db")?;

    conn.transaction::<_, Error, _>(|tx| {
        let new_paths = flash_font::gather_and_clean_font_paths(tx, font_root)?;

        println!("✨ Found {} new fonts to parse.", new_paths.len());

        if new_paths.is_empty() {
            return Ok(());
        }

        // 4. 分块并发解析与落盘
        for chunk in new_paths.chunks(20000) {
            // 并发读取并解析
            let parsed_data: Vec<(String, Vec<String>)> = chunk
                .par_iter()
                .filter_map(|path| {
                    let data_file = open_for_mmap(path).ok()?;
                    let data = unsafe { memmap2::Mmap::map(&data_file).ok()? };
                    let families = parser::get_font_family_names(&data);

                    Some((path.clone(), families.into_iter().collect()))
                })
                .collect();

            // 写入数据库
            for (path, families) in parsed_data {
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
        }

        Ok(())
    })?;

    println!("✅ Database fully updated!");
    Ok(())
}
