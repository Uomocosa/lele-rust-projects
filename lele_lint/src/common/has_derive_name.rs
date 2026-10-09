pub(crate) fn has_derive_name(struct_def: &syn::ItemStruct, names: &[&str]) -> bool {
    struct_def.attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        let syn::Meta::List(list) = &attr.meta else {
            return false;
        };
        list.tokens.to_string().split(',').any(|t| {
            let compact: String = t.split_whitespace().collect();
            names
                .iter()
                .any(|n| compact == *n || compact.ends_with(&format!("::{n}")))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::has_derive_name;

    #[test]
    fn test_usage() {
        let pathed: syn::ItemStruct = syn::parse_quote! {
            #[derive(Debug, serde::Serialize)] struct Cfg { a: u8 }
        };
        assert!(has_derive_name(&pathed, &["Serialize"]));
        assert!(!has_derive_name(&pathed, &["Deref"]));
    }
}
