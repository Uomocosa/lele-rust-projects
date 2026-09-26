pub(crate) fn has_atomic_delegates(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "atomic_delegates")
    })
}

#[cfg(test)]
mod tests {
    use super::has_atomic_delegates;

    #[test]
    fn test_usage() {
        let with: syn::ItemImpl = syn::parse_quote! {
            #[atomic_delegates]
            impl Foo { pub fn f(&self) {} }
        };
        assert!(has_atomic_delegates(&with.attrs));

        let without: syn::ItemImpl = syn::parse_quote! {
            impl Foo { pub fn f(&self) {} }
        };
        assert!(!has_atomic_delegates(&without.attrs));
    }
}
