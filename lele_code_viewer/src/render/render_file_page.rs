use std::borrow::Cow;
use std::collections::HashSet;
use std::path::Path;

use crate::index;
use crate::render;
use crate::source;

const MAX_TEXT_LEN: usize = 524_288;

pub fn render_file_page(
    idx: &index::SymbolIndex,
    file: &Path,
    hl: &source::Highlighter,
    cfg: &render::LinkConfig,
) -> Option<String> {
    match render::file_kind(file) {
        render::FileKind::Rust => render_rust(idx, file, hl, cfg),
        render::FileKind::Text => render_text(idx, file, cfg),
        render::FileKind::Image | render::FileKind::Video | render::FileKind::Audio => {
            render_embed(idx, file, cfg)
        }
        render::FileKind::Other => render_download(idx, file, cfg),
    }
}

// needed helper: rust source with symbol links and recent-change markers
fn render_rust(
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
    body.push_str(&source::render_html(
        idx,
        &text,
        occurrences,
        hl,
        cfg,
        &changed,
    ));
    Some(render::page_shell(cfg, &rel, &body))
}

// needed helper: highlighted text for non-rust files with truncation
fn render_text(idx: &index::SymbolIndex, file: &Path, cfg: &render::LinkConfig) -> Option<String> {
    let text: Cow<'_, str> = idx
        .files
        .get(file)
        .map(|indexed| Cow::Borrowed(indexed.as_str()))
        .or_else(|| read_text_file(idx, file).map(Cow::Owned))?;
    let rel = file.to_string_lossy();
    let mut body = format!("<h1 class=\"path\">{}</h1>", render::escape(&rel));
    let changed = recent_lines(cfg, &rel);
    if let Some(first) = changed.iter().min() {
        body.push_str(&format!(
            "<p class=\"changed-note\"><a href=\"#L{first}\">{} line(s) changed recently</a></p>",
            changed.len()
        ));
    }
    let (display, truncated) = truncate(&text);
    if truncated {
        body.push_str("<p class=\"muted\">truncated: showing the first 512 KiB</p>");
    }
    let ext = file
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    let file_hl = source::highlighter_for_extension(ext);
    let empty: Vec<index::Occurrence> = Vec::new();
    body.push_str(&source::render_html(
        idx, &display, &empty, &file_hl, cfg, &changed,
    ));
    Some(render::page_shell(cfg, &rel, &body))
}

// needed helper: inline preview for image, video and audio files
fn render_embed(idx: &index::SymbolIndex, file: &Path, cfg: &render::LinkConfig) -> Option<String> {
    let rel = file.to_string_lossy();
    let full = render::resolve_in_root(&idx.root, file)?;
    let raw = render::href(cfg, render::LinkKind::Raw, &rel);
    let tag = match render::file_kind(file) {
        render::FileKind::Image => {
            format!(
                "<p><img src=\"{raw}\" alt=\"{}\"></p>",
                render::escape(&rel)
            )
        }
        render::FileKind::Video => {
            format!("<p><video controls preload=\"metadata\" src=\"{raw}\"></video></p>")
        }
        _ => {
            format!("<p><audio controls preload=\"metadata\" src=\"{raw}\"></audio></p>")
        }
    };
    let mut body = format!("<h1 class=\"path\">{}</h1>", render::escape(&rel));
    body.push_str(&tag);
    if let Ok(meta) = std::fs::metadata(&full)
        && let Some(size) = file_size(&meta)
    {
        body.push_str(&format!("<p class=\"muted\">{size}</p>"));
    }
    body.push_str(&format!("<p><a href=\"{raw}\">open raw</a></p>"));
    Some(render::page_shell(cfg, &rel, &body))
}

// needed helper: download page for files with no inline preview
fn render_download(
    idx: &index::SymbolIndex,
    file: &Path,
    cfg: &render::LinkConfig,
) -> Option<String> {
    let rel = file.to_string_lossy();
    let full = render::resolve_in_root(&idx.root, file)?;
    let raw = render::href(cfg, render::LinkKind::Raw, &rel);
    let mut body = format!("<h1 class=\"path\">{}</h1>", render::escape(&rel));
    if let Ok(meta) = std::fs::metadata(&full)
        && let Some(size) = file_size(&meta)
    {
        body.push_str(&format!("<p class=\"muted\">{size}</p>"));
    }
    body.push_str(&format!("<p><a href=\"{raw}\">open raw</a></p>"));
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

// needed helper: read a text file from disk inside the project root
fn read_text_file(idx: &index::SymbolIndex, file: &Path) -> Option<String> {
    let full = render::resolve_in_root(&idx.root, file)?;
    std::fs::read_to_string(full).ok()
}

// needed helper: cap text at a char boundary for large files
fn truncate(text: &str) -> (Cow<'_, str>, bool) {
    if text.len() <= MAX_TEXT_LEN {
        return (Cow::Borrowed(text), false);
    }
    let mut end = MAX_TEXT_LEN;
    while text.get(..end).is_none() {
        end = end.saturating_sub(1);
    }
    match text.get(..end) {
        Some(head) => (Cow::Borrowed(head), true),
        None => (Cow::Borrowed(text), false),
    }
}

// needed helper: human size for a file on disk
fn file_size(meta: &std::fs::Metadata) -> Option<String> {
    let len = meta.len();
    if meta.is_file() {
        Some(format!("{len} bytes"))
    } else {
        None
    }
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
        assert!(!page.contains("class=\"chips\""));

        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("build.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "x\n").unwrap();
        std::fs::write(dir.path().join("data.bin"), [0, 159, 146, 150]).unwrap();
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
        assert!(
            render_file_page(&disk, txt, &hl, &cfg)
                .unwrap()
                .contains('x')
        );
        let bin = std::path::Path::new("data.bin");
        assert!(
            render_file_page(&disk, bin, &hl, &cfg)
                .unwrap()
                .contains("open raw")
        );
        let escape = std::path::Path::new("../build.rs");
        assert!(render_file_page(&disk, escape, &hl, &cfg).is_none());
    }

    #[test]
    fn test_image_embeds_raw() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("shot.png"), [137, 80, 78, 71]).unwrap();
        let idx = index::SymbolIndex {
            root: dir.path().to_path_buf(),
            ..index::SymbolIndex::default()
        };
        let hl = source::highlighter_new();
        let cfg = render::LinkConfig {
            prefix: "/p/demo/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        let page = render_file_page(&idx, std::path::Path::new("shot.png"), &hl, &cfg).unwrap();
        assert!(page.contains("<img src=\"/p/demo/raw/shot.png\""));
    }
}
