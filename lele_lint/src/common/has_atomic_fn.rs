use crate::common;

pub(crate) fn has_atomic_fn(attrs: &[syn::Attribute]) -> bool {
    common::has_attr(attrs, "atomic_delegate")
}

#[cfg(test)]
mod tests {
    use super::has_atomic_fn;

    #[test]
    fn test_usage() {
        let with: syn::ImplItemFn = syn::parse_quote! {
            #[atomic_delegate(Foo)]
            fn check(&self) {}
        };
        assert!(has_atomic_fn(&with.attrs));

        let plural: syn::ImplItemFn = syn::parse_quote! {
            #[atomic_delegates]
            fn check(&self) {}
        };
        assert!(!has_atomic_fn(&plural.attrs));

        let without: syn::ImplItemFn = syn::parse_quote! {
            fn check(&self) {}
        };
        assert!(!has_atomic_fn(&without.attrs));
    }
}
