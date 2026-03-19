use super::schema::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

#[derive(Insertable)]
#[diesel(table_name = font_files)]
pub struct FontFile {
    pub path: String,
}

#[derive(Insertable)]
#[diesel(table_name = font_family_names)]
pub struct FontFamilyName {
    pub file_id: i32,
    pub name: String,
}

// 映射临时表的插入结构（使用生命周期避免不必要的 String 克隆）
#[derive(Insertable)]
#[diesel(table_name = current_disk_paths)]
pub struct TempFontPath<'a> {
    pub path: &'a str,
}

// 用于原生 SQL 查询返回的结构
#[derive(QueryableByName)]
pub struct PathResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub path: String,
}

pub fn establish_connection(db_url: &str) -> Result<SqliteConnection, Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(db_url)?;

    // 表1：存储唯一的文件路径
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS font_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL
            )",
    )
    .execute(&mut conn)?;

    // 表2：存储文件对应的所有 family_name
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS font_family_names (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id INTEGER NOT NULL REFERENCES font_files(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                UNIQUE(file_id, name)
            )",
    )
    .execute(&mut conn)?;

    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_font_files_path ON font_files(path)")
        .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_font_family_names_file_id ON font_family_names(file_id)",
    )
    .execute(&mut conn)?;

    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_font_names_name ON font_family_names(name)")
        .execute(&mut conn)?;

    Ok(conn)
}
