use chardetng::EncodingDetector;
use std::{collections::HashSet, fs, io};

// ── 编码探测 ──────────────────────────────────────────────────────────────────

fn read_text_auto(path: &str) -> io::Result<String> {
    let bytes = fs::read(path)?;
    let mut det = EncodingDetector::new();
    det.feed(&bytes, true);
    Ok(det.guess(None, true).decode(&bytes).0.into_owned())
}

// ── ASS 段落类型 ──────────────────────────────────────────────────────────────

#[derive(Default, PartialEq)]
enum Section {
    Styles,
    Events,
    #[default]
    Other,
}

impl Section {
    fn from_header(s: &str) -> Self {
        match s {
            "[V4+ Styles]" | "[V4 Styles]" | "[V4++ Styles]" => Self::Styles,
            "[Events]" => Self::Events,
            _ => Self::Other,
        }
    }
}

// ── 列格式（懒解析，仅在遇到 Format: 行时更新）──────────────────────────────

#[derive(Clone)]
struct ColFormat {
    /// 目标列在逗号分割后的索引
    target_index: usize,
    /// splitn 的 n（仅 Events 有意义）
    columns_count: usize,
}

impl ColFormat {
    fn style_default() -> Self {
        // Style Format: Name, Fontname, ...
        Self {
            target_index: 1,
            columns_count: usize::MAX,
        }
    }

    fn event_default() -> Self {
        // Event Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
        Self {
            target_index: 9,
            columns_count: 10,
        }
    }

    /// 从 "Format: col0, col1, ..." 行更新自身，needle 为要定位的列名
    fn update_from_format_line(&mut self, format_str: &str, needle: &str) {
        let cols: Vec<&str> = format_str.split(',').map(str::trim).collect();
        self.columns_count = cols.len();
        self.target_index = cols
            .iter()
            .position(|c| c.eq_ignore_ascii_case(needle))
            .unwrap_or(self.columns_count.saturating_sub(1));
    }
}

// ── 核心提取 ──────────────────────────────────────────────────────────────────

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

// ── 段落处理 ──────────────────────────────────────────────────────────────────

fn handle_style(line: &str, fmt: &mut ColFormat, fonts: &mut HashSet<String>) {
    if let Some(rest) = line.strip_prefix("Format:") {
        fmt.update_from_format_line(rest, "fontname");
    } else if let Some(rest) = line.strip_prefix("Style:")
        && let Some(name) = rest.split(',').nth(fmt.target_index)
    {
        add_font(fonts, name);
    }
}

fn handle_event(line: &str, fmt: &mut ColFormat, fonts: &mut HashSet<String>) {
    if let Some(rest) = line.strip_prefix("Format:") {
        fmt.update_from_format_line(rest, "text");
    }
    // Comment Omitted, Dialogue only
    else if matches!(line.split_once(':'), Some(("Dialogue", _))) {
        let content = line[line.find(':').unwrap() + 1..].trim_start();
        if let Some(text) = content.splitn(fmt.columns_count, ',').nth(fmt.target_index) {
            scan_inline_tags(text, fonts);
        }
    }
}

// ── 内联标签扫描 `{\fnName\b1...}` ──────────────────────────────────────────

fn scan_inline_tags(text: &str, fonts: &mut HashSet<String>) {
    // 把 text 视作切片，每次找到 {...} 就处理，然后推进
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

// ── 工具函数 ──────────────────────────────────────────────────────────────────

/// 去掉空白和 `@`（垂直排版标记）后插入
fn add_font(fonts: &mut HashSet<String>, raw: &str) {
    let name = raw.trim().strip_prefix('@').unwrap_or(raw.trim());
    if !name.is_empty() {
        fonts.insert(name.to_owned());
    }
}

// ── 入口 ──────────────────────────────────────────────────────────────────────

fn main() {
    let file_path = r"F:\Coding\rs-projects\lab\star.ass";

    match read_text_auto(file_path) {
        Ok(text) => {
            let fonts = extract_fonts(&text);
            println!("共找到 {} 个字体:", fonts.len());
            fonts.iter().for_each(|f| println!("  • {f}"));
        }
        Err(e) => eprintln!("读取失败: {e}"),
    }
}
