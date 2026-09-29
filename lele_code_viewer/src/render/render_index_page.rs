use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::index;
use crate::render;

pub fn render_index_page(idx: &index::SymbolIndex, cfg: &render::LinkConfig) -> String {
    let mut body = String::new();
    body.push_str(&format!("<h1>{}</h1>", render::escape(&idx.crate_name)));
    body.push_str(&format!(
        "<p class=\"muted\">{} items &middot; {} files</p>",
        idx.items.len(),
        idx.rust_files.len().saturating_add(idx.extra_files.len())
    ));
    if !idx.markdown_files.is_empty() {
        body.push_str("<h2>Docs</h2><ul class=\"list\">");
        for file in &idx.markdown_files {
            let rel = file.to_string_lossy();
            let url = render::href(cfg, render::LinkKind::Md, &rel);
            body.push_str(&format!(
                "<li><a href=\"{url}\">{}</a></li>",
                render::escape(&rel)
            ));
        }
        body.push_str("</ul>");
    }
    body.push_str("<h2>Source</h2>");
    let mut by_dir: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for file in idx.rust_files.iter().chain(idx.extra_files.iter()) {
        let dir = file
            .parent()
            .map_or_else(|| ".".to_string(), |p| p.to_string_lossy().to_string());
        by_dir.entry(dir).or_default().push(file.clone());
    }
    for (dir, files) in by_dir {
        body.push_str(&format!(
            "<h3 class=\"dir\">{}</h3><ul class=\"list\">",
            render::escape(&dir)
        ));
        for file in files {
            let rel = file.to_string_lossy();
            let url = render::href(cfg, render::LinkKind::File, &rel);
            let name = file
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().to_string());
            body.push_str(&format!(
                "<li><a href=\"{url}\">{}</a></li>",
                render::escape(&name)
            ));
        }
        body.push_str("</ul>");
    }
    render::page_shell(cfg, &idx.crate_name, &body)
}

#[cfg(test)]
mod tests {
    use super::render_index_page;
    use crate::index;
    use crate::render;

    #[test]
    fn test_usage() {
        let idx = index::SymbolIndex {
            crate_name: "demo".to_string(),
            rust_files: vec![std::path::PathBuf::from("src/lib.rs")],
            ..index::SymbolIndex::default()
        };
        let cfg = render::LinkConfig {
            prefix: "/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        let html = render_index_page(&idx, &cfg);
        assert!(html.contains("demo"));
        assert!(html.contains("/file/src/lib.rs"));
    }
}
