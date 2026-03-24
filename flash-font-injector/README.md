# flash-font-injector

[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE-MIT)

`flash-font-injector` is a low-level Rust library for temporarily loading system fonts. It provides a reliable mechanism to inject physical font files (e.g., `.ttf`, `.otf`) into the Windows system font table, making them available to local processes without permanently installing them.

Used by [flash-font](https://github.com/naganohara-yoshino/flash-font).

## Features

- **Temporary Injection**: Safely loads fonts using native Windows API (`AddFontResourceExW`).
- **Parallel Loading**: Supports mass loading and unloading of fonts concurrently using `rayon`.
- **RAII Unloading**: Provides a `FontManager` that can optionally unload all loaded fonts automatically when it is dropped, preventing system font table pollution.

## Usage

```rust
use flash_font_injector::{FontManager, FontManagerConfig};
use camino::Utf8PathBuf;

// By default, FontManager keeps fonts loaded after getting dropped.
// We can configure it to clean up the fonts automatically:
let config = FontManagerConfig { keep_loaded_fonts: false };
let mut manager = FontManager::new(config);

// Load a single font
manager.load("path/to/font.ttf".into()).unwrap();

// Load multiple fonts in parallel
let paths = vec![
    Utf8PathBuf::from("path/to/font1.ttf"),
    Utf8PathBuf::from("path/to/font2.ttf"),
];
manager.load_all(paths).unwrap();

// Fonts are unloaded here because keep_loaded_fonts is false
```

## License

MIT OR Apache-2.0