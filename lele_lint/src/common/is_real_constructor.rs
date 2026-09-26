use crate::common;

pub(crate) fn is_real_constructor(method: &syn::ImplItemFn) -> bool {
    if method.sig.receiver().is_some() {
        return false;
    }
    if matches!(method.sig.output, syn::ReturnType::Default) {
        return false;
    }
    if common::is_delegate_call(&method.block) {
        return false;
    }
    if common::is_methods_dispatch(&method.block) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::is_real_constructor;

    #[test]
    fn test_usage() {
        let new: syn::ImplItemFn = syn::parse_quote! {
            pub fn new() -> Self { Self }
        };
        assert!(is_real_constructor(&new));

        let delegate_new: syn::ImplItemFn = syn::parse_quote! {
            pub fn new() -> Self { foo_new::new() }
        };
        assert!(!is_real_constructor(&delegate_new));

        let receiver: syn::ImplItemFn = syn::parse_quote! {
            fn name(&self) -> &'static str { Self::NAME }
        };
        assert!(!is_real_constructor(&receiver));

        let void: syn::ImplItemFn = syn::parse_quote! {
            fn setup() { let x = 1; }
        };
        assert!(!is_real_constructor(&void));

        let dispatch_static: syn::ImplItemFn = syn::parse_quote! {
            fn make() -> Self { crate::methods::foo::make() }
        };
        assert!(!is_real_constructor(&dispatch_static));
    }
}
