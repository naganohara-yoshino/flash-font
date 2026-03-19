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

pub fn initialize_db_connection(
    db_url: &str,
) -> Result<SqliteConnection, Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(db_url)?;

    diesel::sql_query("PRAGMA journal_mode = WAL;").execute(&mut conn)?;
    diesel::sql_query("PRAGMA foreign_keys = ON;").execute(&mut conn)?;
    diesel::sql_query("PRAGMA synchronous = NORMAL;").execute(&mut conn)?;
    diesel::sql_query("PRAGMA temp_store = MEMORY;").execute(&mut conn)?;
    diesel::sql_query("PRAGMA cache_size=-65536").execute(&mut conn)?; // 64MB cache

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

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_font_family_names_file_id ON font_family_names(file_id)",
    )
    .execute(&mut conn)?;

    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_font_names_name ON font_family_names(name)")
        .execute(&mut conn)?;

    Ok(conn)
}
