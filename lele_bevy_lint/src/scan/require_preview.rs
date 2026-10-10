use std::path::Path;

use lele_lint::Diagnostic;
use lele_lint::Origin;
use lele_lint::Project;

use crate::scan;

#[must_use]
pub fn require_preview(
    project: &Project,
    origin: Origin,
    rel_path: &Path,
    file: &syn::File,
    kind: scan::PreviewKind,
    code: &'static str,
    missing: impl FnOnce() -> Diagnostic,
) -> Vec<Diagnostic> {
    let Some(preview) = scan::collect_previews(file)
        .into_iter()
        .find(|preview| preview.kind == kind)
    else {
        return vec![missing()];
    };
    scan::require_ignored(project, origin, rel_path, &preview, code)
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use lele_lint::Origin;
    use lele_lint::Project;

    use super::require_preview;
    use crate::scan;

    #[test]
    fn test_usage() {
        let project = Project::default();
        let file = syn::parse_str("pub fn setup() {}").unwrap();
        let found = require_preview(
            &project,
            Origin::Src,
            Path::new("a.rs"),
            &file,
            scan::PreviewKind::Png,
            "E029",
            || {
                scan::diag(
                    &project,
                    Origin::Src,
                    Path::new("a.rs"),
                    1,
                    "E029",
                    String::from("no preview"),
                )
            },
        );
        assert_eq!(found.first().map(|diag| diag.code), Some("E029"));
    }

    #[test]
    fn test_usage_ignored_preview_satisfies() {
        let project = Project::default();
        let file = syn::parse_str(
            "#[cfg(test)] mod tests {
                 #[test]
                 #[ignore = \"headed\"]
                 fn root_ui_png_preview() {}
             }",
        )
        .unwrap();
        let found = require_preview(
            &project,
            Origin::Src,
            Path::new("a.rs"),
            &file,
            scan::PreviewKind::Png,
            "E029",
            || {
                scan::diag(
                    &project,
                    Origin::Src,
                    Path::new("a.rs"),
                    1,
                    "E029",
                    String::new(),
                )
            },
        );
        assert!(found.is_empty());
    }
}
