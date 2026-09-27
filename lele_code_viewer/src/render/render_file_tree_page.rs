use std::path::Path;
use std::path::PathBuf;

use crate::index;
use crate::project;
use crate::render;

pub fn render_file_tree_page(idx: &index::SymbolIndex, cfg: &render::LinkConfig) -> String {
    let mut body = format!("<h1>{}</h1>", render::escape(&idx.crate_name));
    body.push_str(&format!(
        "<p class=\"muted\">{} items &middot; {} indexed files</p>",
        idx.items.len(),
        idx.rust_files.len()
    ));
    let root_name = idx
        .root
        .file_name()
        .map_or_else(|| ".".to_string(), |n| n.to_string_lossy().to_string());
    body.push_str(&format!(
        "<div class=\"tree\"><details class=\"dir\" open><summary>{}/</summary>",
        render::escape(&root_name)
    ));
    let recent = recent_of(cfg);
    render_dir(&idx.root, &idx.root, cfg, &recent, &mut body);
    body.push_str("</details></div>");
    render::page_shell(cfg, "Files", &body)
}

// needed helper: recursive filesystem listing as nested collapsible lists
fn render_dir(
    root: &Path,
    dir: &Path,
    cfg: &render::LinkConfig,
    recent: &[&str],
    out: &mut String,
) {
    let depth = dir
        .strip_prefix(root)
        .map_or(0, |rel| rel.components().count());
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
            if project::is_hidden_dir(&name) {
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
        let rel = path
            .strip_prefix(root)
            .map_or_else(|_| name.clone(), |p| p.to_string_lossy().to_string());
        if is_dir {
            let prefix = format!("{rel}/");
            let hot = recent.iter().any(|r| r.starts_with(&prefix));
            out.push_str(&format!(
                "<li><details class=\"dir\" data-key=\"{key}\"{open}><summary>{name}/{dot}</summary>",
                key = render::escape(&rel),
                open = if hot { " open" } else { "" },
                name = render::escape(&name),
                dot = dot(hot)
            ));
            render_dir(root, &path, cfg, recent, out);
            out.push_str("</details></li>");
        } else {
            let hot = recent.contains(&rel.as_str());
            out.push_str(&format!(
                "<li class=\"file{}\">{}{}</li>",
                if hot { " recent" } else { "" },
                file_link(root, &path, &name, cfg),
                dot(hot)
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

// needed helper: recently changed paths from live mode (empty otherwise)
fn recent_of(cfg: &render::LinkConfig) -> Vec<&str> {
    cfg.nav
        .as_ref()
        .and_then(|nav| nav.live.as_ref())
        .map(|live| live.recent.keys().map(String::as_str).collect())
        .unwrap_or_default()
}

// needed helper: small marker for a recently changed file or a dir containing one
fn dot(recent: bool) -> &'static str {
    if recent {
        "<span class=\"dot\"></span>"
    } else {
        ""
    }
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
        assert!(!html.contains("class=\"dot\""));
        let live_cfg = render::LinkConfig {
            nav: Some(render::Nav {
                current: "demo".to_string(),
                name: "demo".to_string(),
                root: String::new(),
                projects: Vec::new(),
                view: render::ViewKind::Files,
                live: Some(render::Live {
                    events: "/p/demo/events".to_string(),
                    version: 1,
                    watch: "*".to_string(),
                    recent: std::collections::HashMap::from([("src/lib.rs".to_string(), vec![1])]),
                }),
            }),
            ..cfg
        };
        let live_html = render_file_tree_page(&idx, &live_cfg);
        assert!(live_html.contains(
            "<details class=\"dir\" data-key=\"src\" open><summary>src/<span class=\"dot\">"
        ));
        assert!(live_html.contains("<li class=\"file recent\">"));
        assert!(html.contains("Files"));
        assert!(html.contains("href=\"/p/demo/file/src/lib.rs\""));
        assert!(html.contains("<details class=\"dir\" open>"));
        assert!(html.contains("<details class=\"dir\" data-key=\"src\"><summary>src/</summary>"));
    }
}
