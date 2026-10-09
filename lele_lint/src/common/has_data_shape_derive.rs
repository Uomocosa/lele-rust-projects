use crate::common;

pub(crate) fn has_data_shape_derive(struct_def: &syn::ItemStruct) -> bool {
    common::has_derive_name(
        struct_def,
        &[
            "Serialize",
            "Deserialize",
            "Parser",
            "Args",
            "Subcommand",
            "ValueEnum",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::has_data_shape_derive;

    #[test]
    fn test_usage() {
        let wire: syn::ItemStruct =
            syn::parse_quote! { #[derive(Parser)] struct Args { root: String } };
        assert!(has_data_shape_derive(&wire));

        let plain: syn::ItemStruct = syn::parse_quote! { #[derive(Clone)] struct X { value: u64 } };
        assert!(!has_data_shape_derive(&plain));
    }
}
