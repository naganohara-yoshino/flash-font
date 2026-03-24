# ass-font

[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE-MIT)

`ass-font` is a fast parser for extracting font names from ASS (Advanced SubStation Alpha) subtitle files. It correctly handles section headers, detects encoded files automatically, and scans both standard font definitions and inline dialogue font overrides (e.g., `\fnFontName`).

Used by [flash-font](https://github.com/naganohara-yoshino/flash-font) to prepare a list of fonts that need to be injected before playback.

## Features

- **Robust Parsing**: Scans styles (`[V4+ Styles]`, etc.) and event (`[Events]`) sections.
- **Inline Tag Support**: Detects and extracts fonts from `\fn` dialogue tags correctly.
- **Encoding Autodetection**: Automatically guesses file text encoding (e.g. Shift-JIS, UTF-8, GBK) to correctly read the subtitle files before parsing.

## Usage

Extracting fonts from an ASS text string:

```rust
use ass_font::extract_fonts;

let ass_content = r#"
[V4+ Styles]
Format: Name, Fontname, Fontsize
Style: Default,Arial,20

[Events]
Format: Layer, Start, End, Style, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,{\fnOpen Sans}This is a test.
"#;

let fonts = extract_fonts(ass_content);
assert_eq!(fonts, vec!["Arial", "Open Sans"]);
```

Or read the file directly using auto-detected encoding:

```rust
use ass_font::read_text_auto;

// read_text_auto automatically detects charset and returns a String
let text = read_text_auto("path/to/subtitle.ass".into()).unwrap();
let fonts = extract_fonts(&text);
```

## License

MIT OR Apache-2.0