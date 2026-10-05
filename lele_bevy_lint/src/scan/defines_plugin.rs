#[must_use]
pub fn defines_plugin(file: &syn::File) -> bool {
    file.items.iter().any(|item| match item {
        syn::Item::Impl(impl_block) => impl_block.trait_.as_ref().is_some_and(|(_, path, _)| {
            path.segments
                .last()
                .is_some_and(|segment| segment.ident == "Plugin")
        }),
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::defines_plugin;

    #[test]
    fn test_usage() {
        assert!(defines_plugin(
            &syn::parse_str("impl Plugin for Foo { fn build(&self, _a: &mut A) {} }").unwrap()
        ));
        assert!(!defines_plugin(
            &syn::parse_str("impl Foo { fn build(&self) {} }").unwrap()
        ));
    }
}
