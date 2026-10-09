use lele_lint::Diagnostic;
use lele_lint::Project;

use crate::checkers;
use crate::scan;

pub fn check(_self: &checkers::bevy_ui_mp4::BevyUiMp4, project: &Project) -> Vec<Diagnostic> {
    let graph = scan::CallGraph::collect(project);
    let mut diags = Vec::new();

    for source in project.sources() {
        let rel_path = source.relative_path;
        let file = source.file;
        if !scan::file_reaches_production(file, &graph) {
            continue;
        }
        let found_drivers = scan::drivers(file);
        if found_drivers.is_empty() {
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
        let Some(preview) = previews.iter().find(|p| p.kind == scan::PreviewKind::Mp4) else {
            diags.push(scan::diag(project, source.origin, rel_path,
                visual.line,
                checkers::bevy_ui_mp4::BevyUiMp4::CODE,
                format!(
                    "file spawns UI `{}` and drives it over time/input ({}) but defines no ignored `*_ui_mp4_preview` test",
                    visual.visual,
                    found_drivers.join(", ")
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
                checkers::bevy_ui_mp4::BevyUiMp4::CODE,
                format!(
                    "`fn {}` must carry `#[ignore]` so the default test suite stays GPU-free",
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
            PathBuf::from("ui/sync.rs"),
            syn::parse_str(
                "pub fn setup(s: &mut S) { s.spawn((Node,)); }
                 pub fn tick(time: Res<Time>) { let _ = time.delta_secs(); }",
            )
            .unwrap(),
        );
        let diags = check(&checkers::bevy_ui_mp4::BevyUiMp4, &project);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "E037");
    }
}
