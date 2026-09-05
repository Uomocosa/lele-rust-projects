pub(crate) fn has_rustfmt_skip(impl_block: &syn::ItemImpl) -> bool {
    impl_block.attrs.iter().any(|a| {
        let segs: Vec<_> = a.path().segments.iter().collect();
        segs.len() == 2
            && segs.first().is_some_and(|s| s.ident == "rustfmt")
            && segs.get(1).is_some_and(|s| s.ident == "skip")
    })
}

#[cfg(test)]
mod tests {
    use super::has_rustfmt_skip;
    use syn::ItemImpl;

    #[test]
    fn test_usage() {
        let skipped: ItemImpl = syn::parse_str(
            "#[rustfmt::skip] impl Foo { pub fn new() -> Self { config_new::new() } }",
        )
        .unwrap();
        assert!(has_rustfmt_skip(&skipped));

        let plain: ItemImpl =
            syn::parse_str("impl Foo { pub fn new() -> Self { config_new::new() } }").unwrap();
        assert!(!has_rustfmt_skip(&plain));

        let other_attr: ItemImpl =
            syn::parse_str("#[allow(dead_code)] impl Foo { pub fn new() {} }").unwrap();
        assert!(!has_rustfmt_skip(&other_attr));
    }
}
