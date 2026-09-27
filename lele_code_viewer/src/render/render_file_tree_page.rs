use std::path::Path;
use std::path::PathBuf;

use crate::index;
use crate::render;

pub fn render_file_tree_page(idx: &index::SymbolIndex, cfg: &render::LinkConfig) -> String {
    let mut body = String::from("<h1>Files</h1>");
    body.push_str(&format!(
        "<p class=\"path muted\">{}</p>",
        render::escape(&idx.root.to_string_lossy())
    ));
    body.push_str("<div class=\"tree\">");
    render_dir(&idx.root, &idx.root, cfg, 0, &mut body);
    body.push_str("</div>");
    render::page_shell(cfg, "Files", &body)
}

// needed helper: recursive filesystem listing as nested collapsible lists
fn render_dir(root: &Path, dir: &Path, cfg: &render::LinkConfig, depth: usize, out: &mut String) {
    if depth > 12 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut items: Vec<(bool, String, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let name = entry.file_name().to_string_lossy().to_string();
        if file_type.is_dir() {
            if skip_dir(&name) {
                continue;
            }
            items.push((true, name, entry.path()));
        } else if file_type.is_file() {
            items.push((false, name, entry.path()));
        }
    }
    items.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    out.push_str("<ul class=\"tree-list\">");
    for (is_dir, name, path) in items {
        if is_dir {
            out.push_str(&format!(
                "<li class=\"dir\"><details open><summary>{}/</summary>",
                render::escape(&name)
            ));
            render_dir(root, &path, cfg, depth.saturating_add(1), out);
            out.push_str("</details></li>");
        } else {
            out.push_str(&format!(
                "<li class=\"file\">{}</li>",
                file_link(root, &path, &name, cfg)
            ));
        }
    }
    out.push_str("</ul>");
}

// needed helper: a clickable or plain row for one file
fn file_link(root: &Path, path: &Path, name: &str, cfg: &render::LinkConfig) -> String {
    let rel = path.strip_prefix(root).map_or_else(
        |_| path.to_string_lossy().to_string(),
        |p| p.to_string_lossy().to_string(),
    );
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => {
            let url = render::href(cfg, render::LinkKind::File, &rel);
            format!("<a href=\"{url}\">{}</a>", render::escape(name))
        }
        Some("md") => {
            let url = render::href(cfg, render::LinkKind::Md, &rel);
            format!("<a href=\"{url}\">{}</a>", render::escape(name))
        }
        _ => format!("<span class=\"muted\">{}</span>", render::escape(name)),
    }
}

// needed helper: dirs never shown in the tree
fn skip_dir(name: &str) -> bool {
    matches!(
        name,
        "target" | ".git" | ".devenv" | "node_modules" | "__OLD__"
    )
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::render_file_tree_page;
    use crate::index;
    use crate::render;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        let idx = index::SymbolIndex {
            root: dir.path().to_path_buf(),
            ..index::SymbolIndex::default()
        };
        let cfg = render::LinkConfig {
            prefix: "/p/demo/".to_string(),
            assets: "/assets/".to_string(),
            html: false,
            nav: None,
        };
        let html = render_file_tree_page(&idx, &cfg);
        assert!(html.contains("Files"));
        assert!(html.contains("lib.rs"));
    }
}
