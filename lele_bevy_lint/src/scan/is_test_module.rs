use crate::scan;

pub fn is_test_module(item: &syn::Item) -> bool {
    let syn::Item::Mod(module) = item else {
        return false;
    };
    scan::is_test_attrs(&module.attrs)
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
