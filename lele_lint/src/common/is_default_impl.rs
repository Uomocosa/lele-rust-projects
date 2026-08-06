pub(crate) fn is_default_impl(impl_block: &syn::ItemImpl) -> bool {
    if let Some((_, trait_path, _)) = &impl_block.trait_ {
        return trait_path
            .segments
            .last()
            .is_some_and(|s| s.ident == "Default");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::is_default_impl;
    use syn::ItemImpl;

    #[test]
    fn test_usage() {
        let default_impl: ItemImpl =
            syn::parse_str("impl Default for Bar { fn default() -> Self { Bar { x: 1 } } }")
                .unwrap();
        assert!(is_default_impl(&default_impl));

        let inherent: ItemImpl =
            syn::parse_str("impl Bar { fn new() -> Self { bar_new::new() } }").unwrap();
        assert!(!is_default_impl(&inherent));

        let other_trait: ItemImpl =
            syn::parse_str("impl Clone for Bar { fn clone(&self) -> Self { Bar } }").unwrap();
        assert!(!is_default_impl(&other_trait));
    }
}
