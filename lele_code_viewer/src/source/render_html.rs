use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::BuildHasher;

use syntect::easy::HighlightLines;
use syntect::highlighting::FontStyle;
use syntect::highlighting::Style;

use crate::index;
use crate::render;
use crate::source;

pub fn render_html<S: BuildHasher>(
    idx: &index::SymbolIndex,
    text: &str,
    occurrences: &[index::Occurrence],
    hl: &source::Highlighter,
    cfg: &render::LinkConfig,
    changed: &HashSet<usize, S>,
) -> String {
    let mut occ_by_line: HashMap<usize, Vec<&index::Occurrence>> = HashMap::new();
    for occ in occurrences {
        occ_by_line.entry(occ.line).or_default().push(occ);
    }
    let mut highlighter = hl
        .theme
        .as_ref()
        .map(|theme| HighlightLines::new(&hl.syntax, theme));
    let total = text.split('\n').count().max(1);
    let digits = total.ilog10().saturating_add(1);
    let gutter = digits.saturating_add(2);
    let mut out = format!("<div class=\"code\" style=\"--ln-w:{gutter}ch\">");
    for (idx0, line) in text.split('\n').enumerate() {
        let lineno = idx0.saturating_add(1);
        let chars = styled_chars(line, &mut highlighter, hl);
        let occs = occ_by_line.get(&lineno);
        out.push_str(&render_line(
            idx,
            lineno,
            &chars,
            occs,
            cfg,
            changed.contains(&lineno),
        ));
    }
    out.push_str("</div>");
    out
}

// needed helper: style each character of a line (fallback to uncolored on failure)
fn styled_chars(
    line: &str,
    highlighter: &mut Option<HighlightLines>,
    hl: &source::Highlighter,
) -> Vec<(char, Option<Style>)> {
    if let Some(running) = highlighter
        && let Ok(ranges) = running.highlight_line(line, &hl.syntax_set)
    {
        let mut chars = Vec::new();
        for (style, segment) in ranges {
            for ch in segment.chars() {
                chars.push((ch, Some(style)));
            }
        }
        return chars;
    }
    line.chars().map(|c| (c, None)).collect()
}

// needed helper: file anchor for a symbol id, pointing at its definition block
fn file_href(idx: &index::SymbolIndex, cfg: &render::LinkConfig, target: &str) -> Option<String> {
    let &i = idx.by_id.get(target)?;
    let item = idx.items.get(i)?;
    let rel = item.file.to_string_lossy();
    let base = render::href(cfg, render::LinkKind::File, &rel);
    Some(format!("{base}#L{}-L{}", item.start_line, item.end_line))
}

// needed helper: emit one line with clickable anchors kept inside style spans
fn render_line(
    idx: &index::SymbolIndex,
    lineno: usize,
    chars: &[(char, Option<Style>)],
    occs: Option<&Vec<&index::Occurrence>>,
    cfg: &render::LinkConfig,
    changed: bool,
) -> String {
    let mut opens: HashMap<usize, String> = HashMap::new();
    let mut link_closes: HashMap<usize, usize> = HashMap::new();
    let mut span_closes: HashMap<usize, usize> = HashMap::new();
    let empty: Vec<&index::Occurrence> = Vec::new();
    for occ in occs.unwrap_or(&empty) {
        if occ.start_col >= occ.end_col {
            continue;
        }
        if occ.is_self {
            opens.insert(occ.start_col, String::from("<span class=\"def\">"));
            let entry = span_closes.entry(occ.end_col).or_insert(0);
            *entry = entry.saturating_add(1);
            continue;
        }
        if let Some(target) = &occ.target
            && let Some(href) = file_href(idx, cfg, target)
        {
            opens.insert(occ.start_col, format!("<a class=\"ref\" href=\"{href}\">"));
            let entry = link_closes.entry(occ.end_col).or_insert(0);
            *entry = entry.saturating_add(1);
        }
    }
    let class = if changed { "line changed" } else { "line" };
    let mut out = format!(
        "<div class=\"{class}\" id=\"L{lineno}\"><span class=\"ln\">{lineno}</span><span class=\"cl\">"
    );
    let mut span_open = false;
    let mut current = String::new();
    for (i, (ch, style)) in chars.iter().enumerate() {
        let css = style.as_ref().map_or_else(String::new, style_css);
        let link_closing = link_closes.get(&i);
        let span_closing = span_closes.get(&i);
        if span_open && (link_closing.is_some() || span_closing.is_some() || css != current) {
            out.push_str("</span>");
            span_open = false;
            current.clear();
        }
        if let Some(count) = link_closing {
            for _ in 0..*count {
                out.push_str("</a>");
            }
        }
        if let Some(count) = span_closing {
            for _ in 0..*count {
                out.push_str("</span>");
            }
        }
        if let Some(open) = opens.get(&i) {
            out.push_str(open);
        }
        if !css.is_empty() && !span_open {
            out.push_str("<span style=\"");
            out.push_str(&css);
            out.push_str("\">");
            span_open = true;
            current.clone_from(&css);
        }
        out.push_str(&render::escape(&ch.to_string()));
    }
    if span_open {
        out.push_str("</span>");
    }
    if let Some(count) = link_closes.get(&chars.len()) {
        for _ in 0..*count {
            out.push_str("</a>");
        }
    }
    if let Some(count) = span_closes.get(&chars.len()) {
        for _ in 0..*count {
            out.push_str("</span>");
        }
    }
    out.push_str("</span></div>");
    out
}

// needed helper: inline CSS for a syntect style
fn style_css(style: &Style) -> String {
    let color = style.foreground;
    let mut css = format!("color:#{:02x}{:02x}{:02x};", color.r, color.g, color.b);
    if style.font_style.contains(FontStyle::BOLD) {
        css.push_str("font-weight:700;");
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        css.push_str("font-style:italic;");
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        css.push_str("text-decoration:underline;");
    }
    css
}

#[cfg(test)]
mod tests {
    use super::render_html;
    use crate::index;
    use crate::render;
    use crate::source;

    #[test]
    fn test_usage() {
        let hl = source::highlighter_new();
        let cfg = render::LinkConfig {
            prefix: "/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        let idx = index::SymbolIndex::default();
        let html = render_html(
            &idx,
            "fn main() {}\n",
            &[],
            &hl,
            &cfg,
            &std::collections::HashSet::from([1]),
        );
        assert!(html.contains("<div class=\"line changed\" id=\"L1\">"));
        assert!(html.contains("--ln-w:"));
        assert!(html.contains("fn"));
    }

    #[test]
    fn test_symbol_links_to_file_anchor() {
        let hl = source::highlighter_new();
        let cfg = render::LinkConfig {
            prefix: "/p/demo/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        let mut idx = index::SymbolIndex::default();
        idx.items.push(index::IndexItem {
            id: "a::f".to_string(),
            name: "f".to_string(),
            module: "a".to_string(),
            kind: index::ItemKind::Fn,
            file: std::path::PathBuf::from("src/a.rs"),
            start_line: 3,
            end_line: 5,
            name_line: 3,
            name_col_start: 7,
            name_col_end: 8,
            signature: "pub fn f".to_string(),
            doc: None,
            delegates_to: None,
            is_test: false,
            external: Vec::new(),
        });
        idx.by_id.insert("a::f".to_string(), 0);
        let occs = vec![index::Occurrence {
            line: 1,
            start_col: 0,
            end_col: 1,
            target: Some("a::f".to_string()),
            is_self: false,
        }];
        let html = render_html(
            &idx,
            "f\n",
            &occs,
            &hl,
            &cfg,
            &std::collections::HashSet::new(),
        );
        assert!(html.contains("/p/demo/file/src/a.rs#L3-L5"));
        assert!(!html.contains("/item/"));
    }

    #[test]
    fn test_self_renders_span_not_link() {
        let hl = source::highlighter_new();
        let cfg = render::LinkConfig {
            prefix: "/p/demo/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        let mut idx = index::SymbolIndex::default();
        idx.items.push(index::IndexItem {
            id: "a::call".to_string(),
            name: "call".to_string(),
            module: "a".to_string(),
            kind: index::ItemKind::Fn,
            file: std::path::PathBuf::from("src/a.rs"),
            start_line: 1,
            end_line: 3,
            name_line: 1,
            name_col_start: 7,
            name_col_end: 11,
            signature: "pub fn call".to_string(),
            doc: None,
            delegates_to: None,
            is_test: false,
            external: Vec::new(),
        });
        idx.by_id.insert("a::call".to_string(), 0);
        let occs = vec![index::Occurrence {
            line: 1,
            start_col: 7,
            end_col: 11,
            target: Some("a::call".to_string()),
            is_self: true,
        }];
        let html = render_html(
            &idx,
            "pub fn call() {}\n",
            &occs,
            &hl,
            &cfg,
            &std::collections::HashSet::new(),
        );
        assert!(html.contains("class=\"def\""));
        assert!(html.contains("call</span>"));
        assert!(!html.contains("<a "));
    }
}
