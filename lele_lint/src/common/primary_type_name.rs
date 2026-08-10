pub(crate) fn primary_type_name(file: &syn::File, file_stem: &str) -> Option<String> {
    file.items.iter().find_map(|item| {
        let ident = match item {
            syn::Item::Struct(s) => &s.ident,
            syn::Item::Enum(e) => &e.ident,
            _ => return None,
        };
        let name = ident.to_string();
        if super::to_snake_case(&name) == file_stem {
            Some(name)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::primary_type_name;

    #[test]
    fn test_usage() {
        let file: syn::File =
            syn::parse_str("pub struct AtomicFile;\nimpl AtomicFile { pub fn check(&self) {} }")
                .unwrap();
        assert_eq!(
            primary_type_name(&file, "atomic_file"),
            Some("AtomicFile".to_string())
        );

        let file2: syn::File = syn::parse_str("pub struct Args;\n").unwrap();
        assert_eq!(primary_type_name(&file2, "main"), None);

        let file3: syn::File = syn::parse_str(
            "pub struct FreenetClient;\nimpl FreenetClient { pub fn connect(&self) {} }",
        )
        .unwrap();
        assert_eq!(
            primary_type_name(&file3, "freenet_client"),
            Some("FreenetClient".to_string())
        );
    }
}
