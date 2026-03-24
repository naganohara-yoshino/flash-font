use camino::{Utf8Path, Utf8PathBuf};
use walkdir::WalkDir;

pub fn scan_font_directory(root: impl AsRef<Utf8Path>) -> impl Iterator<Item = Utf8PathBuf> {
    WalkDir::new(root.as_ref())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|file_entry| Utf8PathBuf::from_path_buf(file_entry.into_path()).ok())
        .filter(|path| {
            path.extension().is_some_and(|ext| {
                ["ttf", "otf", "ttc"]
                    .iter()
                    .any(|v| ext.eq_ignore_ascii_case(v))
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(ci))]
    fn test_scan_font_directory() {
        let root = Utf8Path::new(r"G:\Data\fonts");
        let paths = scan_font_directory(&root);
        println!("Found {} fonts", paths.count());
    }
}
