pub fn is_test_module(item: &syn::Item) -> bool {
    let syn::Item::Mod(module) = item else {
        return false;
    };
    module.attrs.iter().any(|attr| {
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
    use super::is_test_module;

    #[test]
    fn test_usage() {
        assert!(is_test_module(
            &syn::parse_str("#[cfg(test)] mod tests {}").unwrap()
        ));
        assert!(!is_test_module(&syn::parse_str("mod inner {}").unwrap()));
    }
}
