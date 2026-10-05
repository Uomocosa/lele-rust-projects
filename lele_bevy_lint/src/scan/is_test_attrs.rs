#[must_use]
pub fn is_test_attrs(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && attr
                .parse_nested_meta(|meta| {
                    if meta.path.is_ident("test") {
                        Ok(())
                    } else {
                        Err(meta.error("expected test"))
                    }
                })
                .is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::is_test_attrs;

    #[test]
    fn test_usage() {
        let item: syn::Item = syn::parse_str("#[cfg(test)] mod tests {}").unwrap();
        let syn::Item::Mod(module) = item else {
            panic!("expected mod");
        };
        assert!(is_test_attrs(&module.attrs));

        let plain: syn::Item = syn::parse_str("mod inner {}").unwrap();
        let syn::Item::Mod(module) = plain else {
            panic!("expected mod");
        };
        assert!(!is_test_attrs(&module.attrs));
    }
}
