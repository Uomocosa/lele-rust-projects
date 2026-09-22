pub(crate) fn has_atomic_delegate(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "atomic_delegate")
    })
}

#[cfg(test)]
mod tests {
    use super::has_atomic_delegate;

    #[test]
    fn test_usage() {
        let with: syn::ItemImpl = syn::parse_quote! {
            #[atomic_delegate]
            impl Foo { pub fn f(&self) {} }
        };
        assert!(has_atomic_delegate(&with.attrs));

        let without: syn::ItemImpl = syn::parse_quote! {
            impl Foo { pub fn f(&self) {} }
        };
        assert!(!has_atomic_delegate(&without.attrs));
    }
}
