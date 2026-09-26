use std::path::Path;

use syn::spanned::Spanned;

use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(
    _self: &checkers::no_dunder_tests::NoDunderTests,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        if !in_dunder(rel_path, project) {
            continue;
        }
        for item in &file.items {
            let Some(line) = cfg_test_line(item) else {
                continue;
            };
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line,
                col: 0,
                code: "E034".to_string(),
                message: "`#[cfg(test)]` is not allowed in a `__basic__` container — cover the function or method that uses these types instead"
                    .to_string(),
                severity: Severity::Error,
            });
        }
    }

    diags
}

// needed helper: whether a path lives inside a whitelisted dunder folder
fn in_dunder(rel_path: &Path, project: &Project) -> bool {
    rel_path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| project.dunder.folders.contains_key(name))
    })
}

// needed helper: cfg(test) item start line, if the item is a test
fn cfg_test_line(item: &syn::Item) -> Option<usize> {
    match item {
        syn::Item::Mod(module) if common::is_cfg_test_mod(module) => {
            Some(module.span().start().line)
        }
        syn::Item::Fn(function) if has_cfg_test(&function.attrs) => {
            Some(function.span().start().line)
        }
        _ => None,
    }
}

// needed helper: cfg(test) attribute detection
fn has_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && matches!(&attr.meta, syn::Meta::List(list) if list.tokens.to_string().contains("test"))
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::no_dunder_tests::NoDunderTests;
    use super::check;
    use crate::Dunder;
    use crate::Project;

    fn project(files: &[(&str, &str)]) -> Project {
        let mut project = Project {
            dunder: Dunder::default(),
            ..Project::default()
        };
        for (path, source) in files {
            let file: syn::File = syn::parse_str(source).unwrap();
            project.parsed_files.insert(PathBuf::from(path), file);
        }
        project
    }

    #[test]
    fn test_usage() {
        let project = project(&[(
            "discovery/__basic__/newtypes.rs",
            "#[cfg(test)] mod tests { #[test] fn test_usage() {} }",
        )]);
        let diags = check(&NoDunderTests, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("__basic__"));
    }

    #[test]
    fn test_usage_allows_tests_outside_dunder() {
        let project = project(&[(
            "discovery/newtypes.rs",
            "#[cfg(test)] mod tests { #[test] fn test_usage() {} }",
        )]);
        assert!(check(&NoDunderTests, &project).is_empty());
    }
}

// no test_usage necessary
