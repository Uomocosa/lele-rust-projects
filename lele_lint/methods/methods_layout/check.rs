use std::path::Path;

use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::EntryKind;
use crate::Project;
use crate::Severity;

pub fn check(
    _self: &checkers::methods_layout::MethodsLayout,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let declared = common::collect_declared(project);

    for (type_snake, declared_type) in &declared {
        for method in &declared_type.methods {
            let rel = Path::new(type_snake).join(format!("{method}.rs"));
            match project.methods_parsed_files.get(&rel) {
                Some(file) => {
                    if !defines_pub_fn(file, method) {
                        diags.push(diag(
                            project,
                            &rel,
                            format!(
                                "`methods/{type_snake}/{method}.rs` must define `pub fn {method}`"
                            ),
                        ));
                    } else if pub_fn_block(file, method).is_some_and(|block| block.stmts.len() <= 1)
                    {
                        diags.push(diag(
                            project,
                            &rel,
                            format!(
                                "`methods/{type_snake}/{method}.rs` body has ≤1 statement — inline `pub fn {method}` in `{type_snake}.rs` under `#[rustfmt::skip]`, then delete this file"
                            ),
                        ));
                    }
                    if !has_test_usage(file) {
                        diags.push(diag(
                            project,
                            &rel,
                            format!(
                                "`methods/{type_snake}/{method}.rs` must carry its own `#[cfg(test)]` `test_usage`"
                            ),
                        ));
                    }
                }
                None => diags.push(diag(
                    project,
                    &rel,
                    format!(
                        "missing `methods/{type_snake}/{method}.rs` for `{type_snake}::{method}` — create it with `pub fn {method}` and a `test_usage` test"
                    ),
                )),
            }
        }

        let mod_rel = Path::new(type_snake).join("mod.rs");
        let expected = common::type_index_content(&declared_type.methods);
        if project.methods_parsed_files.contains_key(&mod_rel) {
            if read_methods_file(project, &mod_rel).as_deref() != Some(expected.as_str()) {
                diags.push(diag(
                    project,
                    &mod_rel,
                    format!(
                        "`methods/{type_snake}/mod.rs` is out of date — run `lele_lint --sync-methods`"
                    ),
                ));
            }
        } else {
            diags.push(diag(
                project,
                &mod_rel,
                format!("missing `methods/{type_snake}/mod.rs` — run `lele_lint --sync-methods`"),
            ));
        }
    }

    for rel in project.methods_parsed_files.keys() {
        if is_index_file(rel) {
            continue;
        }
        let Some(parent) = parent_dir(rel) else {
            continue;
        };
        let Some(stem) = file_stem(rel) else {
            continue;
        };
        if !declared
            .get(&parent)
            .is_some_and(|declared_type| declared_type.methods.contains(&stem))
        {
            diags.push(diag(
                project,
                rel,
                format!(
                    "orphan method file `methods/{parent}/{stem}.rs` — no matching `{parent}::{stem}`"
                ),
            ));
        }
    }

    if !declared.is_empty() {
        let root_mod = Path::new("mod.rs");
        let expected = common::root_index_content(&declared);
        if project.methods_parsed_files.contains_key(root_mod) {
            if read_methods_file(project, root_mod).as_deref() != Some(expected.as_str()) {
                diags.push(diag(
                    project,
                    root_mod,
                    "`methods/mod.rs` is out of date — run `lele_lint --sync-methods`".to_string(),
                ));
            }
        } else {
            diags.push(diag(
                project,
                root_mod,
                "missing `methods/mod.rs` — run `lele_lint --sync-methods`".to_string(),
            ));
        }
    }

    diags
}

// needed helper: build a diagnostic anchored to a methods-relative path
fn diag(project: &Project, rel: &Path, message: String) -> Diagnostic {
    let file = project
        .methods_dir
        .as_ref()
        .map_or_else(|| rel.to_path_buf(), |dir| dir.join(rel));
    Diagnostic {
        file,
        line: 1,
        col: 0,
        code: "E030".to_string(),
        message,
        severity: Severity::Error,
    }
}

// needed helper: does the parsed file expose `pub fn <name>`
fn defines_pub_fn(file: &syn::File, name: &str) -> bool {
    file.items.iter().any(|item| {
        matches!(
            item,
            syn::Item::Fn(func)
                if matches!(func.vis, syn::Visibility::Public(_))
                    && func.sig.ident == name
        )
    })
}

// needed helper: the block of the exposed `pub fn <name>` in a method file
fn pub_fn_block<'a>(file: &'a syn::File, name: &str) -> Option<&'a syn::Block> {
    file.items.iter().find_map(|item| match item {
        syn::Item::Fn(func)
            if matches!(func.vis, syn::Visibility::Public(_)) && func.sig.ident == name =>
        {
            Some(&*func.block)
        }
        _ => None,
    })
}

// needed helper: method file carries its own `#[cfg(test)]` `test_usage`
fn has_test_usage(file: &syn::File) -> bool {
    file.items.iter().any(|item| {
        let syn::Item::Mod(module) = item else {
            return false;
        };
        if !common::is_cfg_test_mod(module) {
            return false;
        }
        module.content.as_ref().is_some_and(|(_, items)| {
            items.iter().any(|inner| {
                matches!(
                    inner,
                    syn::Item::Fn(func) if func.sig.ident == "test_usage"
                )
            })
        })
    })
}

// needed helper: read a methods file from disk by its methods-relative path
fn read_methods_file(project: &Project, rel: &Path) -> Option<String> {
    let entry = project
        .methods_entries
        .iter()
        .find(|entry| entry.relative_path == rel && entry.kind == EntryKind::File)?;
    std::fs::read_to_string(&entry.absolute_path).ok()
}

// needed helper: `mod.rs` files are index files, not method bodies
fn is_index_file(rel: &Path) -> bool {
    file_stem(rel).is_some_and(|stem| stem == "mod")
}

// needed helper: parent directory name of a methods-relative path
fn parent_dir(rel: &Path) -> Option<String> {
    rel.parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .map(str::to_string)
}

// needed helper: file stem of a methods-relative path
fn file_stem(rel: &Path) -> Option<String> {
    rel.file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::check;
    use crate::checkers;
    use crate::checkers::methods_layout::MethodsLayout;
    use crate::Project;

    #[test]
    fn test_usage() {
        let checker = checkers::methods_layout::MethodsLayout;
        let project = Project::default();
        assert!(check(&checker, &project).is_empty());
    }

    #[test]
    fn test_usage_missing_body_names_remediation() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("widget.rs"),
            syn::parse_str(
                "pub struct Widget;\n#[atomic_delegates]\nimpl Widget { pub fn spin(&self) {} }\n",
            )
            .unwrap(),
        );
        let diags = check(&MethodsLayout, &project);
        let body = diags
            .iter()
            .find(|d| d.message.contains("methods/widget/spin.rs"))
            .expect("missing-body diagnostic");
        assert!(body.message.contains("pub fn spin"));
        assert!(body.message.contains("test_usage"));
    }

    #[test]
    fn test_usage_short_body_demands_inline() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("widget.rs"),
            syn::parse_str(
                "pub struct Widget;\n#[atomic_delegates]\nimpl Widget { pub fn spin(&self) {} }\n",
            )
            .unwrap(),
        );
        project.methods_parsed_files.insert(
            PathBuf::from("widget/spin.rs"),
            syn::parse_str("pub fn spin(w: &mut Widget) { w.spin_once() }\n").unwrap(),
        );
        let diags = check(&MethodsLayout, &project);
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("body has ≤1 statement")),
            "expected inline diagnostic, got {diags:?}"
        );
    }

    #[test]
    fn test_usage_multi_statement_body_is_clean() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("widget.rs"),
            syn::parse_str(
                "pub struct Widget;\n#[atomic_delegates]\nimpl Widget { pub fn spin(&self) {} }\n",
            )
            .unwrap(),
        );
        project.methods_parsed_files.insert(
            PathBuf::from("widget/spin.rs"),
            syn::parse_str(
                "pub fn spin(w: &mut Widget) {\n    let n = w.count;\n    w.count = n + 1;\n}\n",
            )
            .unwrap(),
        );
        let diags = check(&MethodsLayout, &project);
        assert!(!diags
            .iter()
            .any(|d| d.message.contains("body has ≤1 statement")));
    }

    #[test]
    fn test_usage_trait_shell_demands_mirror_file() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\n#[atomic_delegates]\nimpl Checker for Foo { fn check(&self) {} }\n",
            )
            .unwrap(),
        );
        let diags = check(&MethodsLayout, &project);
        let body = diags
            .iter()
            .find(|d| d.message.contains("methods/foo/check.rs"))
            .expect("missing-mirror diagnostic for trait shell method");
        assert!(body.message.contains("pub fn check"));
        assert!(body.message.contains("test_usage"));
    }
}
