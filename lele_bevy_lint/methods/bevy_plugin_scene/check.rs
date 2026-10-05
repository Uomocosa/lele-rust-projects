use lele_lint::Diagnostic;
use lele_lint::Project;

use crate::checkers;
use crate::scan;

pub fn check(
    _self: &checkers::bevy_plugin_scene::BevyPluginScene,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for source in project.sources() {
        let rel_path = source.relative_path;
        let file = source.file;
        if !scan::defines_plugin(file) || !scan::dir_spawns_ui(project, source.origin, rel_path) {
            continue;
        }

        let previews = scan::collect_previews(file);
        let Some(preview) = previews.iter().find(|p| p.kind == scan::PreviewKind::Scene) else {
            diags.push(scan::diag(project, source.origin, rel_path,
                1,
                "E038",
                String::from(
                    "file defines a `Plugin` whose domain spawns UI but has no ignored `*_ui_scene_preview` test routing through `lele_bevy_preview::run`",
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
                "E038",
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
            PathBuf::from("ui/spawn.rs"),
            syn::parse_str("pub fn spawn_ui(s: &mut S) { s.spawn((Node,)); }").unwrap(),
        );
        project.parsed_files.insert(
            PathBuf::from("ui/plugin.rs"),
            syn::parse_str(
                "pub struct MyPlugin;
                 impl Plugin for MyPlugin { fn build(&self, app: &mut A) { } }",
            )
            .unwrap(),
        );
        let diags = check(&checkers::bevy_plugin_scene::BevyPluginScene, &project);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "E038");
    }
}
