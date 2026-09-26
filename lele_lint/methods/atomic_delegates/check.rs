use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub fn check(
    _self: &checkers::atomic_delegates::AtomicDelegates,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        let Some(file_stem) = rel_path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(primary) = common::primary_type_name(file, file_stem) else {
            continue;
        };

        for item in &file.items {
            let syn::Item::Impl(impl_block) = item else {
                continue;
            };

            if common::self_type_last(&impl_block.self_ty).as_deref() != Some(primary.as_str()) {
                continue;
            }

            if common::is_default_impl(impl_block) {
                continue;
            }

            if !has_any_method(impl_block) {
                continue;
            }

            if common::has_cfg_test(&impl_block.attrs) {
                continue;
            }

            let is_trait_impl = impl_block.trait_.is_some();

            let methods_over_three: Vec<&syn::Ident> = impl_block
                .items
                .iter()
                .filter_map(|item| {
                    if let syn::ImplItem::Fn(method) = item {
                        if method.block.stmts.len() > 3 {
                            Some(&method.sig.ident)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            if !methods_over_three.is_empty() && !is_trait_impl {
                let names = names_str(&methods_over_three);
                diags.push(Diagnostic {
                    file: project.src_dir.join(rel_path),
                    line: 1,
                    col: 0,
                    code: "E012".to_string(),
                    message: format!(
                        "method(s) `{names}` have >3 statements — extract each into `<type>_<method>.rs`"
                    ),
                    severity: Severity::Error,
                });
                continue;
            }

            let one_liners: Vec<&syn::ImplItemFn> = impl_block
                .items
                .iter()
                .filter_map(|item| {
                    if let syn::ImplItem::Fn(method) = item {
                        if method.block.stmts.len() == 1 {
                            Some(method)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            if !one_liners.is_empty() {
                if common::has_rustfmt_skip(impl_block)
                    || common::has_atomic_delegates(&impl_block.attrs)
                {
                    for method in &one_liners {
                        if common::is_delegate_call(&method.block) && !is_one_line_body(method) {
                            diags.push(Diagnostic {
                                file: project.src_dir.join(rel_path),
                                line: 1,
                                col: 0,
                                code: "E012".to_string(),
                                message: format!(
                                    "one-liner method `{}` body must be on one line, e.g. `{{ module::func(self) }}`",
                                    method.sig.ident
                                ),
                                severity: Severity::Error,
                            });
                        }
                    }
                } else if !common::is_default_impl(impl_block) {
                    let plain: Vec<&&syn::ImplItemFn> = one_liners
                        .iter()
                        .filter(|m| !common::is_real_constructor(m))
                        .collect();
                    if plain.is_empty() {
                        continue;
                    }
                    let names = plain
                        .iter()
                        .map(|m| m.sig.ident.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let ctors: Vec<String> = one_liners
                        .iter()
                        .filter(|m| common::is_real_constructor(m))
                        .map(|m| m.sig.ident.to_string())
                        .collect();
                    let message = if ctors.is_empty() || is_trait_impl {
                        format!(
                            "one-liner method(s) `{names}` require `#[rustfmt::skip]` on the impl block"
                        )
                    } else {
                        format!(
                            "one-liner method(s) `{names}` share a block with constructor(s) `{}`; move the constructor(s) to their own skip-free block, then add `#[rustfmt::skip]` here",
                            ctors.join(", ")
                        )
                    };
                    diags.push(Diagnostic {
                        file: project.src_dir.join(rel_path),
                        line: 1,
                        col: 0,
                        code: "E012".to_string(),
                        message,
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    diags
}

// needed helper: method presence check in impl block
fn has_any_method(impl_block: &syn::ItemImpl) -> bool {
    impl_block
        .items
        .iter()
        .any(|item| matches!(item, syn::ImplItem::Fn(_)))
}

// needed helper: comma-separated ident list
fn names_str(idents: &[&syn::Ident]) -> String {
    idents
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

// needed helper: one-line body check (placeholder)
fn is_one_line_body(_method: &syn::ImplItemFn) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::check;
    use crate::checkers;
    use crate::Project;

    #[test]
    fn test_usage() {
        let checker = checkers::atomic_delegates::AtomicDelegates;
        let project = Project::default();
        assert!(check(&checker, &project).is_empty());
    }

    #[test]
    fn test_usage_real_one_liner_requires_skip() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Foo { fn name(&self) -> &'static str { Self::NAME } }\n",
            )
            .unwrap(),
        );
        let diags = check(&checkers::atomic_delegates::AtomicDelegates, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("#[rustfmt::skip]"));
    }

    #[test]
    fn test_usage_ctor_one_liner_is_exempt() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str("pub struct Foo;\nimpl Foo { pub fn new() -> Self { Self } }\n")
                .unwrap(),
        );
        assert!(check(&checkers::atomic_delegates::AtomicDelegates, &project).is_empty());
    }

    #[test]
    fn test_usage_ctor_mixed_demands_move_out() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Foo { pub fn new() -> Self { Self } fn name(&self) -> &'static str { Self::NAME } }\n",
            )
            .unwrap(),
        );
        let diags = check(&checkers::atomic_delegates::AtomicDelegates, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("move the constructor"));
    }

    #[test]
    fn test_usage_fn_shell_block_requires_skip() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("foo.rs"),
            syn::parse_str(
                "pub struct Foo;\nimpl Foo { #[atomic_delegate(Foo)] pub fn run(&self) {} fn name(&self) -> &'static str { Self::NAME } }\n",
            )
            .unwrap(),
        );
        let diags = check(&checkers::atomic_delegates::AtomicDelegates, &project);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("#[rustfmt::skip]"));
    }
}
