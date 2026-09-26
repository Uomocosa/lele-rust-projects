use std::collections::HashSet;
use std::path::Path;

use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

const RESERVED_DELEGATE_METHODS: &[&str] = &["new"];

pub fn check(
    _self: &checkers::delegate_macro::DelegateMacro,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for (rel_path, file) in &project.parsed_files {
        let Some(stem) = rel_path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(primary) = common::primary_type_name(file, stem) else {
            continue;
        };
        let annotated_inherent = annotated_inherent_blocks(file, &primary);
        if annotated_inherent > 1 {
            diags.push(diag(
                project,
                rel_path,
                format!(
                    "type `{primary}` has {annotated_inherent} `#[atomic_delegates]` impl blocks — merge into one block"
                ),
            ));
        }
        let mut seen: HashSet<String> = HashSet::new();
        for item in &file.items {
            let syn::Item::Impl(impl_block) = item else {
                continue;
            };
            if common::has_cfg_test(&impl_block.attrs) {
                continue;
            }
            if impl_block.trait_.is_some() {
                if common::self_type_last(&impl_block.self_ty).as_deref() != Some(primary.as_str())
                {
                    continue;
                }
                if common::has_atomic_delegates(&impl_block.attrs) {
                    for method in fn_methods(impl_block) {
                        let ident = &method.sig.ident;
                        if is_reserved(ident) {
                            diags.push(diag(
                                project,
                                rel_path,
                                format!(
                                    "`{ident}` is a constructor and must be defined in the struct file, not delegated"
                                ),
                            ));
                        }
                        if common::has_atomic_fn(&method.attrs) {
                            diags.push(diag(
                                project,
                                rel_path,
                                format!(
                                    "`{ident}` carries a redundant `#[atomic_delegate]` — the whole block is already `#[atomic_delegates]`"
                                ),
                            ));
                        }
                        if !seen.insert(format!("{primary}::{ident}")) {
                            diags.push(duplicate_diag(project, rel_path, &primary, ident));
                        }
                    }
                    continue;
                }
                check_fn_shells(
                    project, rel_path, &primary, impl_block, &mut seen, &mut diags,
                );
                let migratable = migratable_idents(impl_block);
                if migratable.is_empty() {
                    continue;
                }
                let type_snake = common::to_snake_case(&primary);
                let mirrors = migratable
                    .iter()
                    .map(|ident| format!("methods/{type_snake}/{ident}.rs"))
                    .collect::<Vec<_>>()
                    .join(", ");
                diags.push(diag(
                    project,
                    rel_path,
                    format!(
                        "trait impl for `{primary}` has unannotated shells; annotate each with `#[atomic_delegate({primary})]` or the whole block with `#[atomic_delegates]`; empty each shell body and move it to {mirrors} (one `pub fn` + `test_usage` per file)"
                    ),
                ));
                continue;
            }
            if common::self_type_last(&impl_block.self_ty).as_deref() != Some(primary.as_str()) {
                continue;
            }
            if !has_fn(impl_block) {
                continue;
            }

            if common::has_atomic_delegates(&impl_block.attrs) {
                for method in fn_methods(impl_block) {
                    let ident = &method.sig.ident;
                    if is_reserved(ident) {
                        diags.push(diag(
                            project,
                            rel_path,
                            format!(
                                "`{ident}` is a constructor and must be defined in the struct file, not delegated"
                            ),
                        ));
                    }
                    if common::has_atomic_fn(&method.attrs) {
                        diags.push(diag(
                            project,
                            rel_path,
                            format!(
                                "`{ident}` carries a redundant `#[atomic_delegate]` — the whole block is already `#[atomic_delegates]`"
                            ),
                        ));
                    }
                    if !seen.insert(format!("{primary}::{ident}")) {
                        diags.push(duplicate_diag(project, rel_path, &primary, ident));
                    }
                }
                continue;
            }

            check_fn_shells(
                project, rel_path, &primary, impl_block, &mut seen, &mut diags,
            );

            if all_methods_are_real_reserved(impl_block) {
                continue;
            }
            let migratable = migratable_idents(impl_block);
            if migratable.is_empty() {
                continue;
            }
            let type_snake = common::to_snake_case(&primary);
            let mirrors = migratable
                .iter()
                .map(|ident| format!("methods/{type_snake}/{ident}.rs"))
                .collect::<Vec<_>>()
                .join(", ");
            let real = real_idents(impl_block);
            if real.is_empty() {
                diags.push(diag(
                    project,
                    rel_path,
                    format!(
                        "inherent impl for `{primary}` must be annotated with `#[atomic_delegates]`; empty each body and move it to {mirrors} (one `pub fn` + `test_usage` per file)"
                    ),
                ));
            } else {
                let kept = real
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                diags.push(diag(
                    project,
                    rel_path,
                    format!(
                        "inherent impl for `{primary}` mixes shells with hand-written methods; split it into a plain block keeping {kept} and an `#[atomic_delegates]` block with empty bodies moved to {mirrors} (one `pub fn` + `test_usage` per file)"
                    ),
                ));
            }
        }
    }
    diags
}

// needed helper: build an E032 diagnostic anchored to a src file
fn diag(project: &Project, rel_path: &Path, message: String) -> Diagnostic {
    Diagnostic {
        file: project.src_dir.join(rel_path),
        line: 1,
        col: 0,
        code: "E032".to_string(),
        message,
        severity: Severity::Error,
    }
}

// needed helper: duplicate-shell diagnostic for a twice-declared (type, method)
fn duplicate_diag(
    project: &Project,
    rel_path: &Path,
    primary: &str,
    ident: &syn::Ident,
) -> Diagnostic {
    diag(
        project,
        rel_path,
        format!("`{ident}` is declared twice for `{primary}` (duplicate `#[atomic_delegates]` shell) — merge into one block"),
    )
}

// needed helper: count `#[atomic_delegates]` inherent impl blocks for a type
fn annotated_inherent_blocks(file: &syn::File, primary: &str) -> usize {
    file.items
        .iter()
        .filter(|item| match item {
            syn::Item::Impl(block) => {
                block.trait_.is_none()
                    && !common::has_cfg_test(&block.attrs)
                    && common::has_atomic_delegates(&block.attrs)
                    && common::self_type_last(&block.self_ty).as_deref() == Some(primary)
            }
            _ => false,
        })
        .count()
}

// needed helper: impl block contains at least one function
fn has_fn(impl_block: &syn::ItemImpl) -> bool {
    impl_block
        .items
        .iter()
        .any(|item| matches!(item, syn::ImplItem::Fn(_)))
}

// needed helper: migratable (empty-or-delegate-shaped, unannotated) method idents
fn migratable_idents(impl_block: &syn::ItemImpl) -> Vec<&syn::Ident> {
    impl_block
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(method)
                if !common::has_atomic_fn(&method.attrs)
                    && (method.block.stmts.is_empty()
                        || common::is_delegate_call(&method.block)
                        || common::is_methods_dispatch(&method.block)) =>
            {
                Some(&method.sig.ident)
            }
            _ => None,
        })
        .collect()
}

// needed helper: hand-written (non-empty, non-delegate, unannotated) method idents
fn real_idents(impl_block: &syn::ItemImpl) -> Vec<&syn::Ident> {
    impl_block
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(method)
                if !common::has_atomic_fn(&method.attrs)
                    && !method.block.stmts.is_empty()
                    && !common::is_delegate_call(&method.block)
                    && !common::is_methods_dispatch(&method.block) =>
            {
                Some(&method.sig.ident)
            }
            _ => None,
        })
        .collect()
}

// needed helper: method items of an impl block
fn fn_methods(impl_block: &syn::ItemImpl) -> Vec<&syn::ImplItemFn> {
    impl_block
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(method) => Some(method),
            _ => None,
        })
        .collect()
}

// needed helper: reserved + duplicate checks for per-function atomic shells
fn check_fn_shells(
    project: &Project,
    rel_path: &Path,
    primary: &str,
    impl_block: &syn::ItemImpl,
    seen: &mut HashSet<String>,
    diags: &mut Vec<Diagnostic>,
) {
    for method in fn_methods(impl_block) {
        if !common::has_atomic_fn(&method.attrs) {
            continue;
        }
        let ident = &method.sig.ident;
        if is_reserved(ident) {
            diags.push(diag(
                project,
                rel_path,
                format!(
                    "`{ident}` is a constructor and must be defined in the struct file, not delegated"
                ),
            ));
        }
        if !seen.insert(format!("{primary}::{ident}")) {
            diags.push(duplicate_diag(project, rel_path, primary, ident));
        }
    }
}

// needed helper: reserved delegate method name check
fn is_reserved(ident: &syn::Ident) -> bool {
    RESERVED_DELEGATE_METHODS.contains(&ident.to_string().as_str())
}

// needed helper: every method is a real (non-delegate) reserved constructor
fn all_methods_are_real_reserved(impl_block: &syn::ItemImpl) -> bool {
    let mut has_fn = false;
    for item in &impl_block.items {
        let syn::ImplItem::Fn(method) = item else {
            continue;
        };
        has_fn = true;
        if !is_reserved(&method.sig.ident) {
            return false;
        }
        if method.block.stmts.is_empty() || common::is_delegate_call(&method.block) {
            return false;
        }
    }
    has_fn
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{all_methods_are_real_reserved, check, is_reserved};
    use crate::checkers::delegate_macro::DelegateMacro;
    use crate::Project;

    #[test]
    fn test_usage() {
        let new_ident: syn::Ident = syn::parse_str("new").unwrap();
        let other: syn::Ident = syn::parse_str("increment").unwrap();
        assert!(is_reserved(&new_ident));
        assert!(!is_reserved(&other));

        let real: syn::ItemImpl = syn::parse_quote! {
            impl Foo { pub const fn new() -> Self { Self } }
        };
        assert!(all_methods_are_real_reserved(&real));

        let delegated: syn::ItemImpl = syn::parse_quote! {
            impl Foo { pub fn new() -> Self { foo_new::new() } }
        };
        assert!(!all_methods_are_real_reserved(&delegated));

        let mixed: syn::ItemImpl = syn::parse_quote! {
            impl Foo { pub fn new() -> Self { Self } pub fn other(&self) {} }
        };
        assert!(!all_methods_are_real_reserved(&mixed));
    }

    #[test]
    fn test_usage_flags_bare_trait_shell_but_exempts_real_body() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Checker for Foo { fn check(&self, p: &Project) { checkers::foo_check::check(self, p) } }\n",
            )
            .unwrap(),
        );
        let diags = check(&DelegateMacro, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("methods/foo/check.rs"));

        let mut real = Project::default();
        real.parsed_files.insert(
            PathBuf::from("bar.rs"),
            syn::parse_str(
                "pub struct Bar;\nimpl Visitor for Bar { fn visit(&self, x: u32) { let y = x + 1; visit_child(y) } }\n",
            )
            .unwrap(),
        );
        assert!(check(&DelegateMacro, &real).is_empty());
    }

    #[test]
    fn test_usage_flags_duplicate_annotated_shell() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\n#[atomic_delegates]\nimpl Checker for Foo { fn check(&self) {} }\n#[atomic_delegates]\nimpl Checker for Foo { fn check(&self) {} }\n",
            )
            .unwrap(),
        );
        let diags = check(&DelegateMacro, &project);
        assert!(diags.iter().any(|d| d.message.contains("duplicate")));
    }

    #[test]
    fn test_usage_flags_multiple_annotated_inherent_blocks() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\n#[atomic_delegates]\nimpl Foo { pub fn run(&self) {} }\n#[atomic_delegates]\nimpl Foo { pub fn stop(&self) {} }\n",
            )
            .unwrap(),
        );
        let diags = check(&DelegateMacro, &project);
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("merge into one block")),
            "expected merge diagnostic, got {diags:?}"
        );
    }

    #[test]
    fn test_usage_single_annotated_block_is_clean() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\n#[atomic_delegates]\nimpl Foo { pub fn run(&self) {} pub fn stop(&self) {} }\n",
            )
            .unwrap(),
        );
        assert!(!check(&DelegateMacro, &project)
            .iter()
            .any(|d| d.message.contains("merge into one block")));
    }

    #[test]
    fn test_usage_fn_shell_in_bare_trait_block_is_clean() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Checker for Foo { fn name(&self) -> &'static str { Self::NAME } #[atomic_delegate(Foo)] fn check(&self) {} }\n",
            )
            .unwrap(),
        );
        assert!(check(&DelegateMacro, &project).is_empty());
    }

    #[test]
    fn test_usage_fn_shell_reserved_new_is_flagged() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Foo { #[atomic_delegate(Foo)] pub fn new() -> Self {} }\n",
            )
            .unwrap(),
        );
        let diags = check(&DelegateMacro, &project);
        assert!(diags.iter().any(|d| d.message.contains("constructor")));
    }

    #[test]
    fn test_usage_cfg_test_impl_is_skipped() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\n#[cfg(test)]\nimpl Foo { fn help(&self) { foo_help::help(self) } }\n",
            )
            .unwrap(),
        );
        assert!(check(&DelegateMacro, &project).is_empty());
    }

    #[test]
    fn test_usage_redundant_fn_attr_in_annotated_block() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\n#[atomic_delegates]\nimpl Foo { #[atomic_delegate(Foo)] pub fn run(&self) {} }\n",
            )
            .unwrap(),
        );
        let diags = check(&DelegateMacro, &project);
        assert!(diags.iter().any(|d| d.message.contains("redundant")));
    }

    #[test]
    fn test_usage_inherent_mixed_demands_split() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Foo { fn run(&self) { foo_run::run(self) } fn compute(&self) -> i32 { let x = 1; x + 1 } }\n",
            )
            .unwrap(),
        );
        let diags = check(&DelegateMacro, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("split"));
        assert!(diags[0].message.contains("methods/foo/run.rs"));
    }

    #[test]
    fn test_usage_all_fn_shell_inherent_is_clean() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Foo { #[atomic_delegate(Foo)] pub fn run(&self) {} }\n",
            )
            .unwrap(),
        );
        assert!(check(&DelegateMacro, &project).is_empty());
    }

    #[test]
    fn test_usage_bare_trait_methods_dispatch_is_flagged() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Checker for Foo { fn check(&self) { crate::methods::foo::check(self) } }\n",
            )
            .unwrap(),
        );
        let diags = check(&DelegateMacro, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("methods/foo/check.rs"));
    }

    #[test]
    fn test_usage_bare_trait_message_suggests_fn_attr() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Checker for Foo { fn check(&self, p: &Project) { checkers::foo_check::check(self, p) } }\n",
            )
            .unwrap(),
        );
        let diags = check(&DelegateMacro, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("#[atomic_delegate("));
    }

    #[test]
    fn test_usage_message_names_mirror_files() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Foo { pub fn run(&self) {} pub fn stop(&self) {} }\n",
            )
            .unwrap(),
        );
        let diags = check(&DelegateMacro, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("methods/foo/run.rs"));
        assert!(diags[0].message.contains("methods/foo/stop.rs"));
    }
}

// no test_usage necessary
