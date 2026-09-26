use std::path::Path;

use crate::checkers;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub fn check(
    _self: &checkers::no_cross_domain_reexport::NoCrossDomainReexport,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (mod_rs_path, info) in &project.module_info {
        let own_domain = own_domain_of(mod_rs_path);

        for reexport in &info.reexports {
            if let Some(target_domain) = reexport_target_domain(&reexport.segments) {
                if target_domain != own_domain && !target_domain.is_empty() {
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
                        code: "E004".to_string(),
                        message: format!(
                            "cross-domain re-export `pub use {}` in mod.rs, move to lib.rs",
                            reexported_path
                        ),
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    diags
}

fn reexport_target_domain(segments: &[String]) -> Option<String> {
    if segments.len() >= 2 && segments.first().is_some_and(|s| s == "crate") {
        return segments.get(1).cloned();
    }
    None
}

// needed helper: top-level domain owning a mod.rs
fn own_domain_of(mod_rs_path: &Path) -> String {
    mod_rs_path
        .components()
        .next()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::own_domain_of;
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

    #[test]
    fn test_usage_own_domain_uses_top_level_segment() {
        assert_eq!(
            own_domain_of(Path::new("discovery/room_peers/mod.rs")),
            "discovery"
        );
        assert_eq!(own_domain_of(Path::new("player/mod.rs")), "player");
    }
}
