use lele_lint::Diagnostic;
use lele_lint::Project;

use crate::checkers;
use crate::scan;

pub fn check(
    _self: &checkers::preview_routing::PreviewRouting,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for source in project.sources() {
        let rel_path = source.relative_path;
        let file = source.file;
        for preview in scan::collect_previews(file) {
            if preview.calls_run {
                continue;
            }
            diags.push(scan::diag(project, source.origin, rel_path,
                preview.line,
                "E039",
                format!(
                    "`fn {}` is a preview test but does not call `lele_bevy_preview::run(...)`; route it through the harness so an empty frame fails",
                    preview.name
                ),
            ));
        }
    }
    diags
}

#[cfg(test)]
mod tests {
    use super::check;
    use crate::checkers;
    use lele_lint::Project;
    use std::path::PathBuf;

    #[test]
    fn test_usage() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("ui/root.rs"),
            syn::parse_str(
                "#[cfg(test)]
                 mod tests {
                     #[test]
                     fn spawn_root_ui_png_preview() { assert!(true); }
                 }",
            )
            .unwrap(),
        );
        let diags = check(&checkers::preview_routing::PreviewRouting, &project);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "E039");
    }
}
