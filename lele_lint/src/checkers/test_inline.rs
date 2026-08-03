use crate::checker::Checker;
use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::entry_kind::EntryKind;
use crate::project::Project;
use crate::severity::Severity;

use super::test_inline_register;

pub struct TestInline;

impl Checker for TestInline {
    fn name(&self) -> &'static str {
        "test_inline"
    }

    fn code(&self) -> &'static str {
        "E007"
    }

    fn check(&self, project: &Project) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for entry in &project.entries {
            if entry.kind != EntryKind::Directory {
                continue;
            }
            if has_tests_component(&entry.relative_path) {
                diags.push(Diagnostic {
                    file: entry.absolute_path.clone(),
                    line: 1,
                    col: 0,
                    code: "E007".to_string(),
                    message: format!(
                        "unit tests must be in the same file as the primary item — delete `{}` and move the tests inline",
                        entry.relative_path.display()
                    ),
                    severity: Severity::Error,
                });
            }
        }

        diags
    }
}

#[rustfmt::skip]
impl TestInline {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        test_inline_register::register(checkers, config)
    }
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
