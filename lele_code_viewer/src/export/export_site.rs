use std::path::Path;
use std::path::PathBuf;

use crate::Error;
use crate::assets;
use crate::index;
use crate::render;
use crate::source;

pub fn export_site(idx: &index::SymbolIndex, out: &Path) -> Result<(), Error> {
    std::fs::create_dir_all(out)?;
    write_page(out, Path::new("index.html"), &render_index(idx))?;

    let hl = source::highlighter_new();
    for file in &idx.rust_files {
        let rel = file.to_string_lossy();
        let page_rel = PathBuf::from(format!("file/{rel}.html"));
        let cfg = export_cfg(&page_rel);
        let html = render::render_file_page(idx, file, &hl, &cfg);
        if let Some(html) = html {
            write_page(out, &page_rel, &html)?;
        }
    }
    for file in &idx.markdown_files {
        let rel = file.to_string_lossy();
        let page_rel = PathBuf::from(format!("md/{rel}.html"));
        let cfg = export_cfg(&page_rel);
        if let Some(html) = render::render_md_page(idx, file, &cfg) {
            write_page(out, &page_rel, &html)?;
        }
    }
    for file in &idx.extra_files {
        if !matches!(
            render::file_kind(file),
            render::FileKind::Text | render::FileKind::Rust
        ) {
            copy_raw(idx, out, file)?;
        }
        let rel = file.to_string_lossy();
        let page_rel = PathBuf::from(format!("file/{rel}.html"));
        let cfg = export_cfg(&page_rel);
        let html = render::render_file_page(idx, file, &hl, &cfg);
        if let Some(html) = html {
            write_page(out, &page_rel, &html)?;
        }
    }

    let assets = out.join("assets");
    std::fs::create_dir_all(&assets)?;
    std::fs::write(assets.join("style.css"), assets::style_css())?;
    std::fs::write(assets.join("app.js"), assets::app_js())?;
    Ok(())
}

// needed helper: render the index page with file-style links
fn render_index(idx: &index::SymbolIndex) -> String {
    let cfg = render::LinkConfig {
        prefix: String::new(),
        assets: String::new(),
        html: true,
        nav: None,
    };
    render::render_index_page(idx, &cfg)
}

// needed helper: relative link prefix for a page's output path
fn export_cfg(page_rel: &Path) -> render::LinkConfig {
    let depth = page_rel.parent().map_or(0, |p| p.components().count());
    let prefix = "../".repeat(depth);
    render::LinkConfig {
        prefix: prefix.clone(),
        assets: prefix,
        html: true,
        nav: None,
    }
}

// needed helper: copy a binary file into the exported raw/ tree
fn copy_raw(idx: &index::SymbolIndex, out: &Path, file: &PathBuf) -> Result<(), Error> {
    let full = render::resolve_in_root(&idx.root, file)
        .ok_or_else(|| Error::NotFound(format!("missing file {}", file.to_string_lossy())))?;
    let dest = out.join("raw").join(file);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(full, dest)?;
    Ok(())
}

// needed helper: write a page creating parent directories
fn write_page(out: &Path, rel: &Path, html: &str) -> Result<(), Error> {
    let full = out.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(full, html)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::export_site;
    use crate::index;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let idx = index::SymbolIndex {
            crate_name: "demo".to_string(),
            rust_files: vec![std::path::PathBuf::from("src/lib.rs")],
            files: std::collections::HashMap::from([(
                std::path::PathBuf::from("src/lib.rs"),
                "pub fn f() {}\n".to_string(),
            )]),
            ..index::SymbolIndex::default()
        };
        export_site(&idx, dir.path()).unwrap();
        assert!(dir.path().join("index.html").exists());
        assert!(dir.path().join("assets/style.css").exists());
    }
}
