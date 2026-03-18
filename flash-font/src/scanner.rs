use camino::{Utf8Path, Utf8PathBuf};
use jwalk::{Parallelism, WalkDir};
use std::time::Duration;

pub fn scan_font_directory(font_root: &Utf8Path) -> impl Iterator<Item = Utf8PathBuf> {
    WalkDir::new(font_root)
        .parallelism(Parallelism::RayonDefaultPool {
            busy_timeout: Duration::from_secs(1),
        })
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|file_entry| Utf8PathBuf::from_path_buf(file_entry.path()).ok())
        .filter(|file_path_buf| {
            file_path_buf.extension().is_some_and(|ext| {
                ["ttf", "otf", "ttc"]
                    .iter()
                    .any(|valid_ext| ext.eq_ignore_ascii_case(valid_ext))
            })
        })
}
