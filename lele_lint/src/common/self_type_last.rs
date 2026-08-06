pub(crate) fn self_type_last(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(tp) = ty {
        return tp.path.segments.last().map(|s| s.ident.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::self_type_last;
    use syn::Type;

    #[test]
    fn test_usage() {
        let simple: Type = syn::parse_str("Foo").unwrap();
        assert_eq!(self_type_last(&simple), Some("Foo".to_string()));

        let qualified: Type = syn::parse_str("crate::clicker::Config").unwrap();
        assert_eq!(self_type_last(&qualified), Some("Config".to_string()));

        let tuple: Type = syn::parse_str("(u32, u32)").unwrap();
        assert_eq!(self_type_last(&tuple), None);
    }
}
