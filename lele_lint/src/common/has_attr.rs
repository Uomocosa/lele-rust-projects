pub(crate) fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == name)
    })
}

#[cfg(test)]
mod tests {
    use super::has_attr;

    #[test]
    fn test_usage() {
        let item: syn::ItemImpl = syn::parse_quote! {
            #[atomic_delegates]
            impl Foo { pub fn f(&self) {} }
        };
        assert!(has_attr(&item.attrs, "atomic_delegates"));
        assert!(!has_attr(&item.attrs, "other"));
    }
}
