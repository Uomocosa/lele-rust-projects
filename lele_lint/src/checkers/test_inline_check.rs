use super::test_inline::TestInline;
use crate::diagnostic;
use crate::entry_kind;
use crate::project;
use crate::severity;

pub(crate) fn check(_self: &TestInline, project: &project::Project) -> Vec<diagnostic::Diagnostic> {
    let mut diags = Vec::new();

    for entry in &project.entries {
        if entry.kind != entry_kind::EntryKind::Directory {
            continue;
        }
        if has_tests_component(&entry.relative_path) {
            diags.push(diagnostic::Diagnostic {
                file: entry.absolute_path.clone(),
                line: 1,
                col: 0,
                code: "E007".to_string(),
                message: format!(
                    "unit tests must be in the same file as the primary item — delete `{}` and move the tests inline",
                    entry.relative_path.display()
                ),
                severity: severity::Severity::Error,
            });
        }
    }

    diags
}

fn has_tests_component(path: &std::path::Path) -> bool {
    path.components()
        .any(|c| c.as_os_str().to_str().is_some_and(|s| s == "tests"))
}

#[cfg(test)]
mod tests {
    use super::has_tests_component;
    use std::path::Path;

    #[test]
    fn test_usage() {
        assert!(has_tests_component(Path::new("player/tests")));
        assert!(has_tests_component(Path::new("player/tests/helpers.rs")));
        assert!(!has_tests_component(Path::new("player/mod.rs")));
        assert!(!has_tests_component(Path::new("bevy_systems")));
    }
}
