use crate::common;

pub(crate) fn is_cfg_test_mod(module: &syn::ItemMod) -> bool {
    common::has_cfg_test(&module.attrs)
}

#[cfg(test)]
mod tests {
    use super::is_cfg_test_mod;
    use syn::ItemMod;

    #[test]
    fn test_usage() {
        let cfg_test: ItemMod = syn::parse_str("#[cfg(test)] mod tests {}").unwrap();
        assert!(is_cfg_test_mod(&cfg_test));

        let plain: ItemMod = syn::parse_str("mod tests {}").unwrap();
        assert!(!is_cfg_test_mod(&plain));

        let other_cfg: ItemMod = syn::parse_str("#[cfg(unix)] mod platform {}").unwrap();
        assert!(!is_cfg_test_mod(&other_cfg));
    }
}
