# Flash Font Injector

Flash Font Injector is a library for temporarily loading system fonts.  

Used by [Flash Font](https://github.com/naganohara-yoshino/flash-font).

## Usage

```rust
use flash_font_injector::FontManager;

let mut manager = FontManager::new();
manager.load("path/to/font.ttf").unwrap();
```

## License

MIT OR Apache-2.0