use std::path::Path;

use lele_lint::Diagnostic;
use lele_lint::Origin;
use lele_lint::Project;

use crate::scan;

#[must_use]
pub fn require_ignored(
    project: &Project,
    origin: Origin,
    rel_path: &Path,
    preview: &scan::Preview,
    code: &'static str,
) -> Option<Diagnostic> {
    (!preview.ignored).then(|| {
        scan::diag(
            project,
            origin,
            rel_path,
            preview.line,
            code,
            format!(
                "`fn {}` must carry `#[ignore]` so the default test suite stays GPU-free",
                preview.name
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::require_ignored;
    use crate::scan;
    use lele_lint::Origin;
    use lele_lint::Project;

    fn preview(ignored: bool) -> scan::Preview {
        scan::Preview {
            name: String::from("root_ui_png_preview"),
            kind: scan::PreviewKind::Png,
            line: 4,
            ignored,
            calls_run: true,
            idents: Vec::new(),
        }
    }

    #[test]
    fn test_usage() {
        let project = Project::default();
        let path = Path::new("a.rs");
        let diag = require_ignored(&project, Origin::Src, path, &preview(false), "E029");
        let diag = diag.unwrap();
        assert_eq!(diag.line, 4);
        assert!(diag.message.contains("root_ui_png_preview"));
        assert!(require_ignored(&project, Origin::Src, path, &preview(true), "E029").is_none());
    }
}
