use std::borrow::Cow;
use std::collections::HashSet;
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
    let text: Cow<'_, str> = idx
        .files
        .get(file)
        .map(|indexed| Cow::Borrowed(indexed.as_str()))
        .or_else(|| read_unindexed(idx, file).map(Cow::Owned))?;
    let empty: Vec<index::Occurrence> = Vec::new();
    let occurrences = idx.occurrences.get(file).unwrap_or(&empty);
    let rel = file.to_string_lossy();
    let mut body = format!("<h1 class=\"path\">{}</h1>", render::escape(&rel));
    let changed = recent_lines(cfg, &rel);
    if let Some(first) = changed.iter().min() {
        body.push_str(&format!(
            "<p class=\"changed-note\"><a href=\"#L{first}\">{} line(s) changed recently</a></p>",
            changed.len()
        ));
    }
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
    body.push_str(&source::render_html(&text, occurrences, hl, cfg, &changed));
    Some(render::page_shell(cfg, &rel, &body))
}

// needed helper: lines changed by the latest live edit of this file
fn recent_lines(cfg: &render::LinkConfig, rel: &str) -> HashSet<usize> {
    cfg.nav
        .as_ref()
        .and_then(|nav| nav.live.as_ref())
        .and_then(|live| live.recent.get(rel))
        .map(|lines| lines.iter().copied().collect())
        .unwrap_or_default()
}

// needed helper: plain-highlight fallback for .rs files outside the index (build.rs, tests/)
fn read_unindexed(idx: &index::SymbolIndex, file: &Path) -> Option<String> {
    if file.extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return None;
    }
    let full = render::resolve_in_root(&idx.root, file)?;
    std::fs::read_to_string(full).ok()
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

        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("build.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "x\n").unwrap();
        let disk = index::SymbolIndex {
            root: dir.path().to_path_buf(),
            ..index::SymbolIndex::default()
        };
        let build = std::path::Path::new("build.rs");
        assert!(
            render_file_page(&disk, build, &hl, &cfg)
                .unwrap()
                .contains("main")
        );
        let txt = std::path::Path::new("notes.txt");
        assert!(render_file_page(&disk, txt, &hl, &cfg).is_none());
        let escape = std::path::Path::new("../build.rs");
        assert!(render_file_page(&disk, escape, &hl, &cfg).is_none());
    }
}
