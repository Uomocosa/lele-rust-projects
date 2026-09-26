pub(crate) fn has_atomic_fn(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "atomic_delegate")
    })
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
