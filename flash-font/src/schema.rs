use diesel::prelude::*;

table! {
    font_files (id) {
        id -> Integer,
        path -> Text,
    }
}
table! {
    font_family_names (id) {
        id -> Integer,
        file_id -> Integer,
        name -> Text,
    }
}
// 巧妙：在此处直接声明临时表的 Schema，方便 Diesel 进行类型安全的批量插入
table! {
    current_disk_paths (path) {
        path -> Text,
    }
}

joinable!(font_family_names -> font_files (file_id));
allow_tables_to_appear_in_same_query!(font_files, font_family_names);
