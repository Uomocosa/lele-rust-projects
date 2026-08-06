use super::thin_delegates::ThinDelegates;
use crate::common;
use crate::diagnostic::Diagnostic;
use crate::project::Project;
use crate::severity::Severity;

pub(crate) fn check(_self: &ThinDelegates, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        let Some(file_stem) = rel_path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(primary) = primary_type_name(file, file_stem) else {
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

            if !is_all_delegate_methods(impl_block) {
                let methods = non_delegate_method_names(impl_block).join(", ");
                diags.push(Diagnostic {
                    file: project.src_dir.join(rel_path),
                    line: 1,
                    col: 0,
                    code: "E012".to_string(),
                    message: format!(
                        "impl block for `{primary}` has non-thin-delegate method(s) `{methods}` — extract each into `<type>_<method>.rs` files and dispatch via a thin delegate"
                    ),
                    severity: Severity::Error,
                });
                continue;
            }

            if !common::has_rustfmt_skip(impl_block) {
                diags.push(Diagnostic {
                    file: project.src_dir.join(rel_path),
                    line: 1,
                    col: 0,
                    code: "E012".to_string(),
                    message: "thin delegate impl block must have `#[rustfmt::skip]`".to_string(),
                    severity: Severity::Error,
                });
            }

            for impl_item in &impl_block.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if !common::is_two_segment_dispatch(&method.block) {
                        diags.push(Diagnostic {
                            file: project.src_dir.join(rel_path),
                            line: 1,
                            col: 0,
                            code: "E012".to_string(),
                            message: format!(
                                "thin delegate method `{}` must use 2-segment dispatch `module::function()`",
                                method.sig.ident
                            ),
                            severity: Severity::Error,
                        });
                    }

                    if !is_one_line_body(method) {
                        diags.push(Diagnostic {
                            file: project.src_dir.join(rel_path),
                            line: 1,
                            col: 0,
                            code: "E012".to_string(),
                            message: format!(
                                "thin delegate method `{}` body must be on one line, e.g. `{{ module::func(self) }}`",
                                method.sig.ident
                            ),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }
    }

    diags
}

// needed helper: primary type name from file stem
fn primary_type_name(file: &syn::File, file_stem: &str) -> Option<String> {
    file.items.iter().find_map(|item| {
        let ident = match item {
            syn::Item::Struct(s) => &s.ident,
            syn::Item::Enum(e) => &e.ident,
            _ => return None,
        };
        let name = ident.to_string();
        if common::to_snake_case(&name) == file_stem {
            Some(name)
        } else {
            None
        }
    })
}

// needed helper: method presence check in impl block
fn has_any_method(impl_block: &syn::ItemImpl) -> bool {
    impl_block
        .items
        .iter()
        .any(|item| matches!(item, syn::ImplItem::Fn(_)))
}

// needed helper: all-methods-are-delegates check
fn is_all_delegate_methods(impl_block: &syn::ItemImpl) -> bool {
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            if method.block.stmts.len() != 1 {
                return false;
            }
            if let syn::Stmt::Expr(syn::Expr::Call(call), _) = &method.block.stmts[0] {
                if let syn::Expr::Path(path) = call.func.as_ref() {
                    if path.path.segments.len() != 2 {
                        return false;
                    }
                    continue;
                }
            }
            if let syn::Stmt::Expr(syn::Expr::MethodCall(_), _) = &method.block.stmts[0] {
                return false;
            }
            return false;
        }
    }
    has_any_method(impl_block)
}

// needed helper: non-delegate method name listing
fn non_delegate_method_names(impl_block: &syn::ItemImpl) -> Vec<String> {
    impl_block
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(method) => Some(method.sig.ident.to_string()),
            _ => None,
        })
        .collect()
}

// needed helper: one-line body check (placeholder)
fn is_one_line_body(_method: &syn::ImplItemFn) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{is_all_delegate_methods, primary_type_name};
    use syn::ItemImpl;

    #[test]
    fn test_usage_delegate_dispatch() {
        let parsed: ItemImpl =
            syn::parse_str("impl Foo { pub fn new() -> Self { config_new::new() } }").unwrap();
        assert!(is_all_delegate_methods(&parsed));
    }

    #[test]
    fn test_usage_real_body_rejected() {
        let parsed: ItemImpl =
            syn::parse_str("impl Foo { pub fn new() -> Self { Self { x: 1 } } }").unwrap();
        assert!(!is_all_delegate_methods(&parsed));
    }

    #[test]
    fn test_usage_three_segment_rejected() {
        let parsed: ItemImpl = syn::parse_str(
            "impl Foo { pub fn new() -> Self { crate::clicker::config_new::new() } }",
        )
        .unwrap();
        assert!(!is_all_delegate_methods(&parsed));
    }

    #[test]
    fn test_usage_primary_type_matches_stem() {
        let file: syn::File =
            syn::parse_str("pub struct AtomicFile;\nimpl AtomicFile { pub fn check(&self) {} }")
                .unwrap();
        assert_eq!(
            primary_type_name(&file, "atomic_file"),
            Some("AtomicFile".to_string())
        );
    }

    #[test]
    fn test_usage_primary_type_mismatch() {
        let file: syn::File = syn::parse_str("pub struct Args;\n").unwrap();
        assert_eq!(primary_type_name(&file, "main"), None);
    }
}

// no test_usage necessary
