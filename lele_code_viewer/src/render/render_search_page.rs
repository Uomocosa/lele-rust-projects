use crate::index;
use crate::render;

pub fn render_search_page(
    idx: &index::SymbolIndex,
    query: &str,
    cfg: &render::LinkConfig,
) -> String {
    let mut body = String::new();
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        body.push_str("<p class=\"muted\">Type a name or path to search.</p>");
        return render::page_shell(cfg, "search", &body);
    }
    body.push_str(&format!(
        "<p class=\"muted\">results for &ldquo;{}&rdquo;</p>",
        render::escape(query.trim())
    ));
    body.push_str("<ul class=\"list\">");
    let mut shown = 0_usize;
    for file in &idx.rust_files {
        if shown >= 300 {
            break;
        }
        if file.to_string_lossy().to_lowercase().contains(&needle) {
            let rel = file.to_string_lossy();
            let url = render::href(cfg, render::LinkKind::File, &rel);
            body.push_str(&format!(
                "<li class=\"file\"><a href=\"{url}\">{}</a></li>",
                render::escape(&rel)
            ));
            shown = shown.saturating_add(1);
        }
    }
    for item in &idx.items {
        if shown >= 300 {
            break;
        }
        if item.name.to_lowercase().contains(&needle) || item.id.to_lowercase().contains(&needle) {
            let url = render::href(cfg, render::LinkKind::Item, &item.id);
            body.push_str(&format!(
                "<li><a href=\"{url}\">{}</a> <span class=\"muted\">{}</span></li>",
                render::escape(&item.name),
                render::escape(&item.id)
            ));
            shown = shown.saturating_add(1);
        }
    }
    body.push_str("</ul>");
    render::page_shell(cfg, "search", &body)
}

#[cfg(test)]
mod tests {
    use super::render_search_page;
    use crate::index;
    use crate::render;

    #[test]
    fn test_usage() {
        let idx = index::SymbolIndex::default();
        let cfg = render::LinkConfig {
            prefix: "/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        let page = render_search_page(&idx, "", &cfg);
        assert!(page.contains("search"));
    }
}
