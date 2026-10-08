pub(crate) fn is_exposed(vis: &syn::Visibility) -> bool {
    match vis {
        syn::Visibility::Public(_) => true,
        syn::Visibility::Restricted(r) => {
            r.path.segments.len() == 1
                && r.path.segments.first().is_some_and(|s| s.ident == "crate")
        }
        syn::Visibility::Inherited => false,
    }
}

#[cfg(test)]
mod tests {
    use super::is_exposed;

    #[test]
    fn test_usage() {
        let public: syn::Visibility = syn::parse_quote!(pub);
        assert!(is_exposed(&public));

        let restricted: syn::Visibility = syn::parse_quote!(pub(crate));
        assert!(is_exposed(&restricted));

        let inherited: syn::Visibility = syn::Visibility::Inherited;
        assert!(!is_exposed(&inherited));
    }
}
