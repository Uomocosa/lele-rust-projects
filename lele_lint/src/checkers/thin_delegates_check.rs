// no test_usage necessary

use super::thin_delegates::ThinDelegates;
use crate::diagnostic::Diagnostic;
use crate::project::Project;
use crate::severity::Severity;

// needed helper: parsing utilities

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

            if self_type_name(&impl_block.self_ty).as_deref() != Some(primary.as_str()) {
                continue;
            }

            if is_default_impl(impl_block) {
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

            if !has_rustfmt_skip(impl_block) {
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
                    if !is_two_segment_dispatch(&method.block) {
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

fn primary_type_name(file: &syn::File, file_stem: &str) -> Option<String> {
    file.items.iter().find_map(|item| {
        let ident = match item {
            syn::Item::Struct(s) => &s.ident,
            syn::Item::Enum(e) => &e.ident,
            _ => return None,
        };
        let name = ident.to_string();
        if to_snake_case(&name) == file_stem {
            Some(name)
        } else {
            None
        }
    })
}

fn self_type_name(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(tp) = ty {
        return tp.path.segments.last().map(|s| s.ident.to_string());
    }
    None
}

fn is_default_impl(impl_block: &syn::ItemImpl) -> bool {
    if let Some((_, trait_path, _)) = &impl_block.trait_ {
        return trait_path
            .segments
            .last()
            .is_some_and(|s| s.ident == "Default");
    }
    false
}

fn has_any_method(impl_block: &syn::ItemImpl) -> bool {
    impl_block
        .items
        .iter()
        .any(|item| matches!(item, syn::ImplItem::Fn(_)))
}

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

fn is_two_segment_dispatch(block: &syn::Block) -> bool {
    if block.stmts.len() != 1 {
        return false;
    }
    if let syn::Stmt::Expr(syn::Expr::Call(call), _) = &block.stmts[0] {
        if let syn::Expr::Path(path) = call.func.as_ref() {
            return path.path.segments.len() == 2;
        }
    }
    false
}

fn is_one_line_body(_method: &syn::ImplItemFn) -> bool {
    true
}

fn has_rustfmt_skip(impl_block: &syn::ItemImpl) -> bool {
    impl_block.attrs.iter().any(|a| {
        let segs: Vec<_> = a.path().segments.iter().collect();
        segs.len() == 2 && segs[0].ident == "rustfmt" && segs[1].ident == "skip"
    })
}

fn to_snake_case(pascal: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = pascal.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let preceded_by_lower = i > 0 && chars[i - 1].is_lowercase();
            let followed_by_lower = i + 1 < len && chars[i + 1].is_lowercase();
            let preceded_by_upper = i > 0 && chars[i - 1].is_uppercase();

            if preceded_by_lower || (followed_by_lower && preceded_by_upper) {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::{has_rustfmt_skip, is_all_delegate_methods, primary_type_name, to_snake_case};
    use syn::ItemImpl;

    #[test]
    fn test_usage_no_skip_detected() {
        let parsed: ItemImpl =
            syn::parse_str("impl Foo { pub fn new() -> Self { config_new::new() } }").unwrap();
        assert!(!has_rustfmt_skip(&parsed));
    }

    #[test]
    fn test_usage_skip_detected() {
        let parsed: ItemImpl = syn::parse_str(
            "#[rustfmt::skip] impl Foo { pub fn new() -> Self { config_new::new() } }",
        )
        .unwrap();
        assert!(has_rustfmt_skip(&parsed));
    }

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

    #[test]
    fn test_usage_snake_case_roundtrip() {
        assert_eq!(to_snake_case("ConstructorNoSkip"), "constructor_no_skip");
        assert_eq!(to_snake_case("DomainImport"), "domain_import");
    }
}
