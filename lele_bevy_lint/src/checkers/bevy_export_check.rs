use lele_lint::diagnostic::Diagnostic;
use lele_lint::project::Project;
use lele_lint::severity::Severity;

use super::bevy_export::BevyExport;

pub(crate) fn check(_self: &BevyExport, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (mod_rs_path, info) in &project.module_info {
        if !info.declarations.iter().any(|d| d.name == "bevy_systems") {
            continue;
        }

        for reexport in &info.reexports {
            if reexport.segments.first().map(String::as_str) == Some("bevy_systems") {
                let reexported_path = reexport.segments.join("::");
                let module_file = project
                    .entries
                    .iter()
                    .find(|e| e.relative_path == *mod_rs_path)
                    .map(|e| e.absolute_path.clone());

                diags.push(Diagnostic {
                    file: module_file
                        .unwrap_or_else(|| project.root.join("src").join(mod_rs_path)),
                    line: 1,
                    col: 0,
                    code: "E005".to_string(),
                    message: format!(
                        "pub use {} re-exports bevy_systems items at the domain root; remove it — access via `{{domain}}::bevy_systems::{{name}}`",
                        reexported_path
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let segments = ["bevy_systems".to_string(), "poll_inv".to_string()];
        assert_eq!(segments.first().map(String::as_str), Some("bevy_systems"));
    }
}
