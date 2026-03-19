use std::collections::HashSet;
use ttf_parser::{Face, name_id};

pub fn get_font_family_names(font_data: &[u8]) -> HashSet<String> {
    let font_count = ttf_parser::fonts_in_collection(font_data).unwrap_or(1);

    (0..font_count)
        .filter_map(|index| Face::parse(font_data, index).ok())
        .flat_map(|face| {
            face.names()
                .into_iter()
                .filter(|name| name.name_id == name_id::FAMILY)
                .filter_map(|name| name.to_string())
        })
        .collect()
}
