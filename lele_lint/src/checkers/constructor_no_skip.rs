// no test_usage necessary

use crate::checker::{Checker, Diagnostic, Severity};
use crate::config::Config;
use crate::project::Project;

use super::constructor_no_skip_register;
// needed helper: parsing utilities

pub struct ConstructorNoSkip;

impl Checker for ConstructorNoSkip {
    fn name(&self) -> &'static str {
        "constructor_no_skip"
    }

    fn code(&self) -> &'static str {
        "E013"
    }

    fn check(&self, project: &Project) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (rel_path, file) in &project.parsed_files {
            for item in &file.items {
                if let syn::Item::Impl(impl_block) = item {
                    if impl_block.trait_.is_some() && !is_default_impl(impl_block) {
                        continue;
                    }

                    if !has_rustfmt_skip(impl_block) {
                        continue;
                    }

                    if is_thin_delegate(impl_block) {
                        continue;
                    }

                    let type_name = type_name_string(&impl_block.self_ty);
                    let blurb = if is_default_impl(impl_block) {
                        format!("impl Default for {type_name}")
                    } else if has_any_real_constructor(impl_block) {
                        format!("constructor impl for {type_name}")
                    } else {
                        continue;
                    };

                    diags.push(Diagnostic {
                        file: project.src_dir.join(rel_path),
                        line: 1,
                        col: 0,
                        code: "E013".to_string(),
                        message: format!("{blurb} must not have #[rustfmt::skip]",),
                        severity: Severity::Error,
                    });
                }
            }
        }

        diags
    }
}

#[rustfmt::skip]
impl ConstructorNoSkip {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        constructor_no_skip_register::register(checkers, config)
    }
}

fn has_rustfmt_skip(impl_block: &syn::ItemImpl) -> bool {
    impl_block.attrs.iter().any(|a| {
        let segs: Vec<_> = a.path().segments.iter().collect();
        segs.len() == 2 && segs[0].ident == "rustfmt" && segs[1].ident == "skip"
    })
}

fn type_name_string(ty: &syn::Type) -> String {
    if let syn::Type::Path(tp) = ty {
        return tp
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
    }
    quote::quote!(#ty).to_string()
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

fn is_thin_delegate(impl_block: &syn::ItemImpl) -> bool {
    if impl_block.items.is_empty() {
        return false;
    }
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            if !is_single_delegate_call(&method.block) {
                return false;
            }
        }
    }
    true
}

fn is_single_delegate_call(block: &syn::Block) -> bool {
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

fn has_any_real_constructor(impl_block: &syn::ItemImpl) -> bool {
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            let sig = &method.sig;
            if sig.receiver().is_some() {
                continue;
            }
            if matches!(sig.output, syn::ReturnType::Default) {
                continue;
            }
            if is_single_delegate_call(&method.block) {
                continue;
            }
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{has_any_real_constructor, is_default_impl, is_thin_delegate};
    use syn::ItemImpl;

    #[test]
    fn test_usage_detects_default_skip() {
        let code =
            "#[rustfmt::skip] impl Default for Bar { fn default() -> Self { Bar { x: 1 } } }";
        let parsed: ItemImpl = syn::parse_str(code).unwrap();
        assert!(is_default_impl(&parsed));
        assert!(!is_thin_delegate(&parsed));
    }

    #[test]
    fn test_usage_constructor_detection() {
        let parsed: ItemImpl =
            syn::parse_str("impl Foo { fn production() -> Self { Foo { x: 1 } } }").unwrap();
        assert!(has_any_real_constructor(&parsed));
        assert!(!is_thin_delegate(&parsed));
    }

    #[test]
    fn test_usage_thin_delegate_has_no_real_constructor() {
        let parsed: ItemImpl =
            syn::parse_str("impl Foo { pub fn new() -> Self { config_new::new() } }").unwrap();
        assert!(!has_any_real_constructor(&parsed));
        assert!(is_thin_delegate(&parsed));
    }
}
