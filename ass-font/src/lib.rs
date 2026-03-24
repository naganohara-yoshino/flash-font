//! A library for extracting fonts used in ASS (Advanced SubStation Alpha) subtitle files.

use camino::Utf8Path;
use chardetng::EncodingDetector;
use std::{collections::HashSet, fs, io};

/// Reads a file into a `String`, automatically detecting and decoding its character encoding.
pub fn read_text_auto(path: &Utf8Path) -> io::Result<String> {
    let bytes = fs::read(path)?;
    let mut det = EncodingDetector::new();
    det.feed(&bytes, true);
    Ok(det.guess(None, true).decode(&bytes).0.into_owned())
}

// ── ASS Section Types ────────────────────────────────────────────────────────

/// Represents a section in an ASS file.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
enum Section {
    Styles,
    Events,
    #[default]
    Other,
}

impl Section {
    /// Creates a `Section` from a section header string.
    fn from_header(s: &str) -> Self {
        match s {
            "[V4+ Styles]" | "[V4 Styles]" | "[V4++ Styles]" => Self::Styles,
            "[Events]" => Self::Events,
            _ => Self::Other,
        }
    }
}

// ── Column Formatting (Lazy parsing, updated only when a "Format:" line is encountered) ──

/// Represents the column format for a specific section.
#[derive(Debug, PartialEq, Eq, Clone)]
struct ColFormat {
    /// The index of the target column after splitting by commas.
    target_index: usize,
    /// The maximum number of columns expected (used for `splitn`, relevant for Events).
    columns_count: usize,
}

impl ColFormat {
    /// Default format for the Styles section.
    fn style_default() -> Self {
        // Style Format: Name, Fontname, ...
        Self {
            target_index: 1,
            columns_count: usize::MAX,
        }
    }

    /// Default format for the Events section.
    fn event_default() -> Self {
        // Event Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
        Self {
            target_index: 9,
            columns_count: 10,
        }
    }

    /// Updates the format from a "Format: col0, col1, ..." line, calculating the index of the `needle` column.
    fn update_from_format_line(&mut self, format_str: &str, needle: &str) {
        let cols: Vec<&str> = format_str.split(',').map(str::trim).collect();
        self.columns_count = cols.len();
        self.target_index = cols
            .iter()
            .position(|c| c.eq_ignore_ascii_case(needle))
            .unwrap_or(self.columns_count.saturating_sub(1));
    }
}

// ── Core Extraction ──────────────────────────────────────────────────────────

/// Extracts a unique, sorted list of font names used in the provided ASS subtitle content.
pub fn extract_fonts(ass: &str) -> Vec<String> {
    let mut fonts = HashSet::<String>::new();
    let mut section = Section::default();
    let mut style_fmt = ColFormat::style_default();
    let mut event_fmt = ColFormat::event_default();

    for line in ass.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if line.starts_with('[') && line.ends_with(']') {
            section = Section::from_header(line);
            continue;
        }

        match section {
            Section::Styles => handle_style(line, &mut style_fmt, &mut fonts),
            Section::Events => handle_event(line, &mut event_fmt, &mut fonts),
            Section::Other => {}
        }
    }

    let mut out: Vec<_> = fonts.into_iter().collect();
    out.sort();
    out
}

// ── Section Processing ───────────────────────────────────────────────────────

/// Handles a line in the Styles section, updating the format or extracting a font name.
fn handle_style(line: &str, fmt: &mut ColFormat, fonts: &mut HashSet<String>) {
    if let Some(rest) = line.strip_prefix("Format:") {
        fmt.update_from_format_line(rest, "fontname");
    } else if let Some(rest) = line.strip_prefix("Style:")
        && let Some(name) = rest.split(',').nth(fmt.target_index)
    {
        add_font(fonts, name);
    }
}

/// Handles a line in the Events section, updating the format or extracting font tags from dialogue text.
fn handle_event(line: &str, fmt: &mut ColFormat, fonts: &mut HashSet<String>) {
    if let Some(rest) = line.strip_prefix("Format:") {
        fmt.update_from_format_line(rest, "text");
    }
    // Only process Dialogue lines (ignore Comments)
    else if matches!(line.split_once(':'), Some(("Dialogue", _))) {
        let content = line[line.find(':').unwrap() + 1..].trim_start();
        if let Some(text) = content.splitn(fmt.columns_count, ',').nth(fmt.target_index) {
            scan_inline_tags(text, fonts);
        }
    }
}

// ── Inline Tag Scanning `{\fnName\b1...}` ────────────────────────────────────

/// Scans for inline font tags (e.g., `\fnFontName`) within dialogue text and extracts their values.
fn scan_inline_tags(text: &str, fonts: &mut HashSet<String>) {
    // Treat text as a slice, processing each `{...}` block and advancing
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('}') else { break };

        rest[..close]
            .split('\\')
            .filter_map(|tag| tag.strip_prefix("fn"))
            .for_each(|name| add_font(fonts, name));

        rest = &rest[close + 1..];
    }
}

// ── Utility Functions ────────────────────────────────────────────────────────

/// Inserts a font name into the collection, removing whitespace and the `@` prefix for vertical typography if present.
fn add_font(fonts: &mut HashSet<String>, raw: &str) {
    let trimmed = raw.trim();
    let name = trimmed.strip_prefix('@').unwrap_or(trimmed);
    if !name.is_empty() {
        fonts.insert(name.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_fonts_lucky_star() {
        let ass = r"
            [Script Info]
            Title:lucky star
            Original Script:CASO&I.G
            Synch Point:0
            ScriptType:v4.00+
            Collisions:Normal
            PlayResX:704
            PlayResY:396
            Timer:100.0000

            [V4+ Styles]
            Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
            Style: zhengwen,方正准圆_GBK,26,&H00FFFFFF,&H004080FF,&H20B71082,&H99B71082,-1,0,0,0,100,100,0,0.00,0,3,1,2,30,30,8,134
            Style: zhushi,方正准圆_GBK,19,&H00FFFFFF,&H004080FF,&H20E22E1B,&H99E22E1B,-1,0,0,0,100,100,0,0.00,0,3,1,8,30,30,8,134
            Style: jinggao,方正准圆_GBK,17,&H00FFFFFF,&H00FFFFFF,&H00000000,&HFF000000,-1,0,0,0,100,100,0,0.00,1,3,1,8,15,15,5,134
            Style: staff,@DFGKanTeiRyu-XB,18,&H00FFFFFF,&H00000000,&H407A0748,&HA0FFFFFF,0,0,0,0,105,105,1,0.00,1,3,0,7,30,30,10,128
            Style: OPJ,DFGKanTeiRyu-XB,18,&H00FFFFFF,&H00000000,&H208B0C66,&H66666666,0,0,0,0,100,105,2,0.00,1,3,1,8,30,30,10,128
            Style: OPC,方正少儿_GBK,24,&H00FFFFFF,&H00000000,&H208B0C66,&H66666666,0,0,0,0,100,100,0,0.00,1,3,1,2,30,30,7,134
            Style: EDJ,MS Gothic,21,&H00EEEEEE,&H90FFFFFF,&H12333333,&H20000000,-1,0,0,0,100,100,1,0.00,0,3,0,8,30,30,8,128
            Style: EDC,方正黑体_GBK,19,&H00EEEEEE,&HFF000000,&H12333333,&H20000000,0,0,0,0,100,100,2,0.00,0,3,0,8,30,30,12,134

            [Events]
            Format: Layer, Start, End, Style, Actor, MarginL, MarginR, MarginV, Effect, Text
            Dialogue: 0,0:00:06.08,0:00:08.78,OPJ,NTP,0000,0000,0000,,曖昧３センチ　そりゃぷにってコトかい？　ちょっ！
            Dialogue: 0,0:00:08.95,0:00:11.97,OPJ,NTP,0000,0000,0000,,らっぴんぐが制服…だぁぁ不利ってことない　ぷ。
            Dialogue: 0,0:00:12.20,0:00:13.55,OPJ,NTP,0000,0000,0000,,がんばっちゃ　やっちゃっちゃ
            ";
        let fonts_extracted = extract_fonts(ass);
        let mut fonts_expected = vec![
            "DFGKanTeiRyu-XB",
            "MS Gothic",
            "方正少儿_GBK",
            "方正准圆_GBK",
            "方正黑体_GBK",
        ];
        fonts_expected.sort();
        assert_eq!(fonts_extracted, fonts_expected);
    }

    #[test]
    fn test_extract_fonts_make_heroine_ga_oosugiru() {
        let ass = r"
            [Script Info]
            ; Script generated by Aegisub 9706-cibuilds-20caaabc0
            ; http://www.aegisub.org/
            Title: [KitaujiSub] Make Heroine ga Oosugiru! - 12
            ScriptType: v4.00+
            WrapStyle: 2
            ScaledBorderAndShadow: yes
            YCbCr Matrix: TV.709
            PlayResX: 1920
            PlayResY: 1080
            LayoutResX: 1920
            LayoutResY: 1080

            [V4+ Styles]
            Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
            Style: Text - CN,Source Han Sans TC Medium,80,&H00F5F5F5,&H000000FF,&H00252525,&H00000000,0,0,0,0,100,100,1,0,1,2.5,0,2,10,10,52,1
            Style: Text - JP,Source Han Sans Medium,52,&H00F5F5F5,&H000000FF,&H00252525,&H00000000,0,0,0,0,100,100,1,0,1,2.2,0,2,10,10,10,1
            Style: Text - CN - UP,Source Han Sans TC Medium,80,&H00F5F5F5,&H000000FF,&H00252525,&H00000000,0,0,0,0,100,100,1,0,1,2.5,0,8,10,10,10,1
            Style: Text - JP - UP,Source Han Sans Medium,52,&H00F5F5F5,&H000000FF,&H00252525,&H00000000,0,0,0,0,100,100,1,0,1,2.2,0,8,10,10,80,1
            Style: Screen,Source Han Sans TC Medium,80,&H00F5F5F5,&H000000FF,&H00252525,&H00000000,0,0,0,0,100,100,1,0,1,2.5,0,2,10,10,53,1
            Style: Title,华康翩翩体W5-A,72,&H00B9B9B9,&H000000FF,&H00B99A57,&H00B99A57,0,0,0,0,100,100,5,0,1,0,0,2,10,10,285,1
            Style: Ruby,Source Han Sans TC Medium,45,&H00F5F5F5,&H000000FF,&H00252525,&H00000000,0,0,0,0,100,100,2,0,1,2.2,0,2,10,10,115,1
            Style: Staff,华康翩翩体W5-A,60,&H00B9B9B9,&H000000FF,&H00B99A57,&H00B99A57,0,0,0,0,100,100,0,0,1,0,0,8,10,10,30,1

            [Events]
            Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
            Comment: 0,0:00:00.00,0:00:00.00,Note,,0,0,0,,--------------- Subtitle Staff&Title ---------------
            Comment: 0,0:05:41.45,0:05:47.37,Screen,,0,0,0,,# 參見這裡 https://www.nonhoi.jp/amusement/
            Dialogue: 0,0:05:41.44,0:05:41.48,Screen,,0,0,0,,{=0}{\alpha&H00&\an4\fs70\bord0\c&H312C33&\pos(455,330)}兒童列車
            Dialogue: 0,0:05:41.48,0:05:41.52,Screen,,0,0,0,,{=0}{\alpha&H00&\an4\fs70\bord0\c&H312C33&\pos(457.06,310)}兒童列車
            Dialogue: 0,0:05:41.52,0:05:41.57,Screen,,0,0,0,,{=0}{\alpha&H00&\an4\fs70\bord0\c&H312C33&\pos(459.13,290.16)}兒童列車
            Dialogue: 0,0:05:42.78,0:05:42.82,Screen,,0,0,0,,{=2}{\alpha&H00&\an4\fnSource Han Sans TC\fs40\bord0\fsp0\c&H38434A&\pos(175.71,1125.35)}隨著「嘟嘟—」的汽笛聲前進！乘上列車沿著遊樂場內鋪設的軌道周遊，好好探險吧。
            Dialogue: 0,0:05:42.82,0:05:42.86,Screen,,0,0,0,,{=2}{\alpha&H00&\an4\fnSource Han Sans TC\fs40\bord0\fsp0\c&H38434A&\pos(177.77,1105.37)}隨著「嘟嘟—」的汽笛聲前進！乘上列車沿著遊樂場內鋪設的軌道周遊，好好探險吧。
            Dialogue: 0,0:05:42.86,0:05:42.90,Screen,,0,0,0,,{=2}{\alpha&H00&\an4\fnSource Han Sans TC\fs40\bord0\fsp0\c&H38434A&\pos(179.85,1085.56)}隨著「嘟嘟—」的汽笛聲前進！乘上列車沿著遊樂場內鋪設的軌道周遊，好好探險吧。
            Dialogue: 0,0:05:42.90,0:05:42.94,Screen,,0,0,0,,{=2}{\alpha&H00&\an4\fnSource Han Sans TC\fs40\bord0\fsp0\c&H38434A&\pos(181.98,1065.73)}隨著「嘟嘟—」的汽笛聲前進！乘上列車沿著遊樂場內鋪設的軌道周遊，好好探險吧。
            Dialogue: 0,0:05:42.94,0:05:42.98,Screen,,0,0,0,,{=2}{\alpha&H00&\an4\fnSource Han Sans TC\fs40\bord0\fsp0\c&H38434A&\pos(184.07,1045.73)}隨著「嘟嘟—」的汽笛聲前進！乘上列車沿著遊樂場內鋪設的軌道周遊，好好探險吧。
            Dialogue: 0,0:05:42.98,0:05:43.03,Screen,,0,0,0,,{=2}{\alpha&H00&\an4\fnSource Han Sans TC\fs40\bord0\fsp0\c&H38434A&\pos(186.17,1025.88)}隨著「嘟嘟—」的汽笛聲前進！乘上列車沿著遊樂場內鋪設的軌道周遊，好好探險吧。
            Dialogue: 0,0:05:43.03,0:05:43.07,Screen,,0,0,0,,{=2}{\alpha&H00&\an4\fnSource Han Sans TC\fs40\bord0\fsp0\c&H38434A&\pos(188.31,1006.03)}隨著「嘟嘟—」的汽笛聲前進！乘上列車沿著遊樂場內鋪設的軌道周遊，好好探險吧。
            Dialogue: 0,0:05:43.07,0:05:43.11,Screen,,0,0,0,,{=2}{\alpha&H00&\an4\fnSource Han Sans TC\fs40\bord0\fsp0\c&H38434A&\pos(190.43,986.13)}隨著「嘟嘟—」的汽笛聲前進！乘上列車沿著遊樂場內鋪設的軌道周遊，好好探險吧。
";
        let fonts_extracted = extract_fonts(ass);
        let mut fonts_expected = vec![
            "Source Han Sans TC",
            "Source Han Sans Medium",
            "华康翩翩体W5-A",
            "Source Han Sans TC Medium",
        ];
        fonts_expected.sort();
        assert_eq!(fonts_extracted, fonts_expected);
    }

    #[test]
    fn test_scan_inline_tags() {
        let mut fonts = HashSet::new();
        scan_inline_tags("{\\", &mut fonts);
        scan_inline_tags("{fn", &mut fonts);
        scan_inline_tags("{\\fn}", &mut fonts);
        scan_inline_tags("{\\fn\\}", &mut fonts);
        assert!(fonts.is_empty());

        let mut fonts = HashSet::new();
        scan_inline_tags("{\\fnSource Han Sans TC\\b2e}", &mut fonts);
        assert_eq!(fonts, HashSet::from(["Source Han Sans TC".to_string()]));

        let mut fonts = HashSet::new();
        scan_inline_tags("{{\\fnArial}text}", &mut fonts);
        assert_eq!(fonts, HashSet::from(["Arial".to_string()]));
    }
}
