use lele_lint::Diagnostic;
use lele_lint::Origin;
use lele_lint::Project;

use crate::checkers;
use crate::scan;

pub fn check(_self: &checkers::bevy_ui::BevyUi, project: &Project) -> Vec<Diagnostic> {
    let graph = scan::CallGraph::collect(project);
    let mut diags = Vec::new();

    for source in project.sources() {
        let rel_path = source.relative_path;
        let file = source.file;
        if !scan::file_reaches_production(file, &graph) {
            continue;
        }
        let visuals = scan::prod_visuals(file);
        let Some(visual) = visuals
            .iter()
            .find(|found| graph.reaches_production(&found.owner))
        else {
            continue;
        };

        let previews = scan::collect_previews(file);
        let Some(preview) = previews.iter().find(|p| p.kind == scan::PreviewKind::Png) else {
            diags.push(scan::diag(
                project,
                source.origin,
                rel_path,
                visual.line,
                checkers::bevy_ui::BevyUi::CODE,
                format!(
                    "file spawns UI `{}` reachable from production but defines no ignored `*_ui_png_preview` test that renders it through `lele_bevy_preview::run`",
                    visual.visual
                ),
            ));
            continue;
        };

        if !preview.ignored {
            diags.push(scan::diag(
                project,
                source.origin,
                rel_path,
                preview.line,
                checkers::bevy_ui::BevyUi::CODE,
                format!(
                    "`fn {}` must carry `#[ignore]` so the default test suite stays GPU-free",
                    preview.name
                ),
            ));
        }

        substance_check(project, source.origin, rel_path, file, preview, &mut diags);
    }

    diags
}

// needed helper: the preview must touch a component declared in this file
fn substance_check(
    project: &Project,
    origin: Origin,
    rel_path: &std::path::Path,
    file: &syn::File,
    preview: &scan::Preview,
    diags: &mut Vec<Diagnostic>,
) {
    let components = scan::declared_components(file);
    if components.is_empty() {
        return;
    }
    if preview
        .idents
        .iter()
        .any(|ident| components.contains(ident))
    {
        return;
    }
    diags.push(scan::diag(
        project,
        origin,
        rel_path,
        preview.line,
        checkers::bevy_ui::BevyUi::CODE,
        format!(
            "`fn {}` must reference at least one component declared in this file ({}) so the preview cannot be a no-op shell",
            preview.name,
            components.join(", ")
        ),
    ));
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
            syn::parse_str("pub fn setup(s: &mut S) { s.spawn((Node,)); }").unwrap(),
        );
        let diags = check(&checkers::bevy_ui::BevyUi, &project);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "E029");
    }

    #[test]
    fn test_usage_routed_ignored_preview_is_clean() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("ui/root.rs"),
            syn::parse_str(
                "pub fn setup(s: &mut S) { s.spawn((Node,)); }
                 #[cfg(test)]
                 mod tests {
                     use lele_bevy_preview::{run, scene::Scene};
                     #[test]
                     #[ignore = \"headed\"]
                     fn spawn_root_ui_png_preview() {
                         let scene = Scene { name: String::from(\"s\") };
                         let _ = run(&scene, &Config::default(), \"x\");
                     }
                 }",
            )
            .unwrap(),
        );
        assert!(check(&checkers::bevy_ui::BevyUi, &project).is_empty());
    }
}
