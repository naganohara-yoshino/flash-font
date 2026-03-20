# ASS Font

ASS Font is a library for extracting fonts from ASS subtitles.  

Used by [Flash Font](https://github.com/naganohara-yoshino/flash-font).

## Usage

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

```rust
use ass_font::read_text_auto;

let text = read_text_auto("path/to/subtitle.ass").unwrap();
let fonts = extract_fonts(&text);
```

## License

MIT OR Apache-2.0