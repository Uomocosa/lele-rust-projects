// no test_usage necessary

use crate::checker::{Checker, Diagnostic, Severity};
use crate::config::Config;
use crate::project::Project;

use super::thin_delegates_register;
// needed helper: parsing utilities

pub struct ThinDelegates;

impl Checker for ThinDelegates {
    fn name(&self) -> &'static str {
        "thin_delegates"
    }

    fn code(&self) -> &'static str {
        "E012"
    }

    fn check(&self, project: &Project) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (rel_path, file) in &project.parsed_files {
            for item in &file.items {
                if let syn::Item::Impl(impl_block) = item {
                    if impl_block.trait_.is_some() {
                        continue;
                    }
                    if is_constructor_only_impl(impl_block) {
                        continue;
                    }
                    if !is_all_delegate_methods(impl_block) {
                        continue;
                    }

                    if !has_rustfmt_skip(impl_block) {
                        diags.push(Diagnostic {
                            file: project.src_dir.join(rel_path),
                            line: 1,
                            col: 0,
                            code: "E012".to_string(),
                            message: "thin delegate impl block must have `#[rustfmt::skip]`"
                                .to_string(),
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
                        }
                    }
                }
            }
        }

        diags
    }
}

#[rustfmt::skip]
impl ThinDelegates {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        thin_delegates_register::register(checkers, config)
    }
}

fn is_constructor_only_impl(impl_block: &syn::ItemImpl) -> bool {
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            if method.sig.ident == "default" || method.sig.receiver().is_none() {
                continue;
            }
            return false;
        } else {
            return false;
        }
    }
    !impl_block.items.is_empty()
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
    !impl_block.items.is_empty()
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

fn has_rustfmt_skip(impl_block: &syn::ItemImpl) -> bool {
    impl_block.attrs.iter().any(|a| {
        let segs: Vec<_> = a.path().segments.iter().collect();
        segs.len() == 2 && segs[0].ident == "rustfmt" && segs[1].ident == "skip"
    })
}

#[cfg(test)]
mod tests {
    use super::{has_rustfmt_skip, is_all_delegate_methods};
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
    fn test_usage_three_segment_rejected() {
        let parsed: ItemImpl = syn::parse_str(
            "impl Foo { pub fn new() -> Self { crate::clicker::config_new::new() } }",
        )
        .unwrap();
        assert!(!is_all_delegate_methods(&parsed));
    }
}
