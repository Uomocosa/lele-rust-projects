use std::collections::HashMap;

use syntect::easy::HighlightLines;
use syntect::highlighting::FontStyle;
use syntect::highlighting::Style;

use crate::index;
use crate::render;
use crate::source;

pub fn render_html(
    text: &str,
    occurrences: &[index::Occurrence],
    hl: &source::Highlighter,
    cfg: &render::LinkConfig,
) -> String {
    let mut occ_by_line: HashMap<usize, Vec<&index::Occurrence>> = HashMap::new();
    for occ in occurrences {
        occ_by_line.entry(occ.line).or_default().push(occ);
    }
    let mut highlighter = hl
        .theme
        .as_ref()
        .map(|theme| HighlightLines::new(&hl.syntax, theme));
    let mut out = String::from("<div class=\"code\">");
    for (idx0, line) in text.split('\n').enumerate() {
        let lineno = idx0.saturating_add(1);
        let chars = styled_chars(line, &mut highlighter, hl);
        let occs = occ_by_line.get(&lineno);
        out.push_str(&render_line(lineno, &chars, occs, cfg));
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

// needed helper: emit one line with clickable anchors kept inside style spans
fn render_line(
    lineno: usize,
    chars: &[(char, Option<Style>)],
    occs: Option<&Vec<&index::Occurrence>>,
    cfg: &render::LinkConfig,
) -> String {
    let mut opens: HashMap<usize, String> = HashMap::new();
    let mut closes: HashMap<usize, usize> = HashMap::new();
    let empty: Vec<&index::Occurrence> = Vec::new();
    for occ in occs.unwrap_or(&empty) {
        if occ.start_col >= occ.end_col {
            continue;
        }
        if let Some(target) = &occ.target {
            let href = render::href(cfg, render::LinkKind::Item, target);
            let class = if occ.is_self { "ref def" } else { "ref" };
            opens.insert(
                occ.start_col,
                format!("<a class=\"{class}\" href=\"{href}\">"),
            );
            let entry = closes.entry(occ.end_col).or_insert(0);
            *entry = entry.saturating_add(1);
        }
    }
    let mut out = format!(
        "<div class=\"line\" id=\"L{lineno}\"><span class=\"ln\">{lineno}</span><span class=\"cl\">"
    );
    let mut span_open = false;
    let mut current = String::new();
    for (i, (ch, style)) in chars.iter().enumerate() {
        let css = style.as_ref().map_or_else(String::new, style_css);
        let closing = closes.get(&i);
        if span_open && (closing.is_some() || css != current) {
            out.push_str("</span>");
            span_open = false;
            current.clear();
        }
        if let Some(count) = closing {
            for _ in 0..*count {
                out.push_str("</a>");
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
    if let Some(count) = closes.get(&chars.len()) {
        for _ in 0..*count {
            out.push_str("</a>");
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
    use crate::render;
    use crate::source;

    #[test]
    fn test_usage() {
        let hl = source::highlighter_new();
        let cfg = render::LinkConfig {
            prefix: "/".to_string(),
            html: false,
        };
        let html = render_html("fn main() {}\n", &[], &hl, &cfg);
        assert!(html.contains("id=\"L1\""));
        assert!(html.contains("fn"));
    }
}
