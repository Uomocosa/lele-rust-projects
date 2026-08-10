use super::thin_delegates::ThinDelegates;
use crate::common;
use crate::diagnostic;
use crate::project;
use crate::severity;

pub(crate) fn check(
    _self: &ThinDelegates,
    project: &project::Project,
) -> Vec<diagnostic::Diagnostic> {
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
                diags.push(diagnostic::Diagnostic {
                    file: project.src_dir.join(rel_path),
                    line: 1,
                    col: 0,
                    code: "E012".to_string(),
                    message: format!(
                        "method(s) `{names}` have >3 statements — extract each into `<type>_<method>.rs`"
                    ),
                    severity: severity::Severity::Error,
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
                if !common::has_rustfmt_skip(impl_block) {
                    let names = one_liners
                        .iter()
                        .map(|m| m.sig.ident.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    diags.push(diagnostic::Diagnostic {
                        file: project.src_dir.join(rel_path),
                        line: 1,
                        col: 0,
                        code: "E012".to_string(),
                        message: format!(
                            "one-liner method(s) `{names}` require `#[rustfmt::skip]` on the impl block"
                        ),
                        severity: severity::Severity::Error,
                    });
                }

                for method in &one_liners {
                    if !is_one_line_body(method) {
                        diags.push(diagnostic::Diagnostic {
                            file: project.src_dir.join(rel_path),
                            line: 1,
                            col: 0,
                            code: "E012".to_string(),
                            message: format!(
                                "one-liner method `{}` body must be on one line, e.g. `{{ module::func(self) }}`",
                                method.sig.ident
                            ),
                            severity: severity::Severity::Error,
                        });
                    }
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

// no test_usage necessary
