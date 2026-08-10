use std::path::Path;

use super::no_cross_domain_reexport::NoCrossDomainReexport;
use crate::diagnostic;
use crate::project;
use crate::severity;

pub(crate) fn check(
    _self: &NoCrossDomainReexport,
    project: &project::Project,
) -> Vec<diagnostic::Diagnostic> {
    let mut diags = Vec::new();

    for (mod_rs_path, info) in &project.module_info {
        let own_domain = mod_rs_path
            .parent()
            .unwrap_or(Path::new(""))
            .to_string_lossy()
            .to_string();

        for reexport in &info.reexports {
            if let Some(target_domain) = reexport_target_domain(&reexport.segments) {
                if target_domain != own_domain && !target_domain.is_empty() {
                    let reexported_path = reexport.segments.join("::");
                    let module_file = project
                        .entries
                        .iter()
                        .find(|e| e.relative_path == *mod_rs_path)
                        .map(|e| e.absolute_path.clone());

                    diags.push(diagnostic::Diagnostic {
                        file: module_file
                            .unwrap_or_else(|| project.root.join("src").join(mod_rs_path)),
                        line: 1,
                        col: 0,
                        code: "E004".to_string(),
                        message: format!(
                            "cross-domain re-export `pub use {}` in mod.rs, move to lib.rs",
                            reexported_path
                        ),
                        severity: severity::Severity::Error,
                    });
                }
            }
        }
    }

    diags
}

fn reexport_target_domain(segments: &[String]) -> Option<String> {
    if segments.len() >= 2 && segments[0] == "crate" {
        return segments.get(1).cloned();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::reexport_target_domain;

    #[test]
    fn test_usage() {
        assert_eq!(
            reexport_target_domain(&["crate".into(), "bad_positional".into(), "Positional".into()]),
            Some("bad_positional".into())
        );
        assert_eq!(
            reexport_target_domain(&["config".into(), "Config".into()]),
            None
        );
        assert_eq!(reexport_target_domain(&[]), None);
        assert_eq!(
            reexport_target_domain(&["event".into(), "PlayerEvent".into()]),
            None
        );
    }
}
