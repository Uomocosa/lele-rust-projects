use std::path::Path;

use crate::index;
use crate::render;
use crate::source;

pub fn render_file_page(
    idx: &index::SymbolIndex,
    file: &Path,
    hl: &source::Highlighter,
    cfg: &render::LinkConfig,
) -> Option<String> {
    let text = idx.files.get(file)?;
    let empty: Vec<index::Occurrence> = Vec::new();
    let occurrences = idx.occurrences.get(file).unwrap_or(&empty);
    let rel = file.to_string_lossy();
    let mut body = format!("<h1 class=\"path\">{}</h1>", render::escape(&rel));
    if let Some(list) = idx.items_by_file.get(file) {
        body.push_str("<div class=\"chips\">");
        for &i in list {
            let Some(item) = idx.items.get(i) else {
                continue;
            };
            let url = render::href(cfg, render::LinkKind::Item, &item.id);
            body.push_str(&format!(
                "<a class=\"chip\" href=\"{url}\">{}</a>",
                render::escape(&item.name)
            ));
        }
        body.push_str("</div>");
    }
    body.push_str(&source::render_html(text, occurrences, hl, cfg));
    Some(render::page_shell(cfg, &rel, &body))
}

#[cfg(test)]
mod tests {
    use super::render_file_page;
    use crate::index;
    use crate::render;
    use crate::source;

    #[test]
    fn test_usage() {
        let mut idx = index::SymbolIndex::default();
        let file = std::path::PathBuf::from("src/a.rs");
        idx.files
            .insert(file.clone(), "pub fn f() {}\n".to_string());
        let hl = source::highlighter_new();
        let cfg = render::LinkConfig {
            prefix: "/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        let page = render_file_page(&idx, &file, &hl, &cfg).unwrap();
        assert!(page.contains("pub"));
    }
}
