use super::constructor_no_skip::ConstructorNoSkip;
use crate::common;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &ConstructorNoSkip, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        for item in &file.items {
            if let syn::Item::Impl(impl_block) = item {
                if impl_block.trait_.is_some() && !common::is_default_impl(impl_block) {
                    continue;
                }

                if !common::has_rustfmt_skip(impl_block) {
                    continue;
                }

                if is_atomic_delegate(impl_block) {
                    continue;
                }

                let type_name = type_name_string(&impl_block.self_ty);
                let blurb = if common::is_default_impl(impl_block) {
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

// needed helper: type name string extraction
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

// needed helper: atomic delegate body detection
fn is_atomic_delegate(impl_block: &syn::ItemImpl) -> bool {
    if impl_block.items.is_empty() {
        return false;
    }
    if common::is_default_impl(impl_block) {
        return false;
    }
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            if !common::is_delegate_call(&method.block) {
                return false;
            }
        }
    }
    true
}

// needed helper: real constructor detection (non-delegate static method)
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
            if common::is_delegate_call(&method.block) {
                continue;
            }
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{has_any_real_constructor, is_atomic_delegate};
    use syn::ItemImpl;

    #[test]
    fn test_usage_detects_default_skip() {
        let code =
            "#[rustfmt::skip] impl Default for Bar { fn default() -> Self { Bar { x: 1 } } }";
        let parsed: ItemImpl = syn::parse_str(code).unwrap();
        assert!(!is_atomic_delegate(&parsed));
    }

    #[test]
    fn test_usage_constructor_detection() {
        let parsed: ItemImpl = syn::parse_str(
            "impl Foo { fn production() -> Self { let a = 1; let b = 2; let c = 3; Foo { x: a + b + c } } }",
        )
        .unwrap();
        assert!(has_any_real_constructor(&parsed));
        assert!(!is_atomic_delegate(&parsed));
    }

    #[test]
    fn test_usage_atomic_delegate_has_no_real_constructor() {
        let parsed: ItemImpl =
            syn::parse_str("impl Foo { pub fn new() -> Self { config_new::new() } }").unwrap();
        assert!(!has_any_real_constructor(&parsed));
        assert!(is_atomic_delegate(&parsed));
    }

    #[test]
    fn test_usage_struct_literal_is_real_constructor() {
        let parsed: ItemImpl =
            syn::parse_str("impl Foo { pub fn new(x: i32) -> Self { Self { x } } }").unwrap();
        assert!(has_any_real_constructor(&parsed));
        assert!(!is_atomic_delegate(&parsed));
    }

    #[test]
    fn test_usage_self_constructor_call_is_real_constructor() {
        let parsed: ItemImpl =
            syn::parse_str("impl Foo { pub fn coop() -> Self { Self::new() } }").unwrap();
        assert!(has_any_real_constructor(&parsed));
        assert!(!is_atomic_delegate(&parsed));
    }
}

// no test_usage necessary
