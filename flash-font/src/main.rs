use std::fs::File;

use camino::Utf8Path;
use diesel::prelude::*;
use rayon::prelude::*;

use flash_font::{db, parser, schema};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let font_root = Utf8Path::new(r"G:\Data\fonts\超级字体整合包 XZ\精简包\日文");
    let mut conn = db::initialize_db_connection("fonts.db")?;

    // 所有操作包裹在一个事务中，保证 SQLite 在内存中极速执行
    conn.transaction::<_, Box<dyn std::error::Error>, _>(|tx| {
        let new_paths = flash_font::gather_and_clean_font_paths(tx, font_root)?;

        println!("✨ Found {} new fonts to parse.", new_paths.len());

        let mut count = 0;

        // 4. 分块并发解析与落盘 (Chunked Parallelism) - 防止吃光内存
        if !new_paths.is_empty() {
            for chunk in new_paths.chunks(100) {
                // 并发读取并解析这 100 个文件
                let parsed_data: Vec<(String, Vec<String>)> = chunk
                    .par_iter()
                    .filter_map(|path_str| {
                        let data_file = File::open(path_str).ok()?;
                        let data = unsafe { memmap2::Mmap::map(&data_file).ok()? };
                        let families = parser::get_font_family_names(&data);

                        Some((path_str.clone(), families.into_iter().collect()))
                    })
                    .collect();

                // 将这 100 个解析结果写入数据库
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

                count += chunk.len();
                println!("✅ Parsed and inserted {} fonts.", count);
            }
        }

        Ok(())
    })?;

    println!("✅ Database fully updated!");
    Ok(())
}
