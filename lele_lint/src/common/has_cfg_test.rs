pub(crate) fn has_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("cfg") {
            return false;
        }
        match &attr.meta {
            syn::Meta::List(list) => list.tokens.to_string().contains("test"),
            _ => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::has_cfg_test;

    #[test]
    fn test_usage() {
        let test_impl: syn::ItemImpl =
            syn::parse_quote! { #[cfg(test)] impl Foo { fn help(&self) {} } };
        assert!(has_cfg_test(&test_impl.attrs));

        let other_cfg: syn::ItemImpl =
            syn::parse_quote! { #[cfg(unix)] impl Foo { fn help(&self) {} } };
        assert!(!has_cfg_test(&other_cfg.attrs));

        let plain: syn::ItemImpl = syn::parse_quote! { impl Foo { fn help(&self) {} } };
        assert!(!has_cfg_test(&plain.attrs));
    }
}
