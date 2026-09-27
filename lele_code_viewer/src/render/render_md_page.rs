use std::path::Path;

use crate::index;
use crate::markdown;
use crate::render;

pub fn render_md_page(
    idx: &index::SymbolIndex,
    rel: &Path,
    cfg: &render::LinkConfig,
) -> Option<String> {
    let text = std::fs::read_to_string(idx.root.join(rel)).ok()?;
    let body = format!(
        "<article class=\"md\">{}</article>",
        markdown::render_md(&text)
    );
    Some(render::page_shell(cfg, &rel.to_string_lossy(), &body))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::render_md_page;
    use crate::index;
    use crate::render;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("README.md"), "# Hi\n").unwrap();
        let idx = index::SymbolIndex {
            root: dir.path().to_path_buf(),
            ..index::SymbolIndex::default()
        };
        let cfg = render::LinkConfig {
            prefix: "/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        let page = render_md_page(&idx, Path::new("README.md"), &cfg).unwrap();
        assert!(page.contains("<h1>Hi</h1>"));
    }
}
