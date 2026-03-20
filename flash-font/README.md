# flash-font

[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/naganohara-yoshino/flash-font)
[![Cargo](https://img.shields.io/crates/v/flash-font.svg)](https://crates.io/crates/flash-font)
[![Documentation](https://docs.rs/flash-font/badge.svg)](https://docs.rs/flash-font)

A high-performance Rust library for caching font metadata in a SQLite database. Designed for quick scanning of font directories and fast retrieval of font paths by their family names.

## Features

- **Blazing Fast Scanning**: Uses `walkdir` for parallel directory traversal. 
- **Efficient Metadata Extraction**: Utilizes memory-mapped files (`memmap2`) and `ttf-parser` for high-performance font analysis.
- **Smart Synchronization**: Only parses new or modified font files. Stale database entries are automatically cleaned up.
- **Database Backend**: Powered by SQLite and Diesel for reliable, indexed storage.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
flash-font = "0.1"
camino = "1.2"
```

### Example: Updating the Database and Searching

```rust
use flash_font::{update_font_database, select_font_by_name};
use camino::Utf8Path;

fn main() -> anyhow::Result<()> {
    let font_dir = Utf8Path::new("C:/Path/To/Fonts");
    let db_url = "fonts.db";

    // Scan directory and update the database
    let new_fonts_count = update_font_database(font_dir, db_url)?;
    println!("Added {} new fonts to the database.", new_fonts_count);

    // Search for a font path by its family name
    let font_paths = select_font_by_name("Arial", db_url)?;
    for path in font_paths {
        println!("Found Arial at: {}", path);
    }

    Ok(())
}
```

## Architecture

1.  **Scanner**: Recursively scans the target directory for supported font files.
2.  **Cleaner**: Compares disk files with database records, removing orphans and identifying new files.
3.  **Parser**: Extracts font family names in parallel using multiple threads and memory mapping.
4.  **Database**: Stores mappings between file paths and family names in a normalized SQLite schema.



## License

MIT OR Apache-2.0
