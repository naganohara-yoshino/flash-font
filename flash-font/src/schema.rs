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

joinable!(font_family_names -> font_files (file_id));
allow_tables_to_appear_in_same_query!(font_files, font_family_names);
