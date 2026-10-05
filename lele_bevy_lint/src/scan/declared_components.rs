pub fn declared_components(file: &syn::File) -> Vec<String> {
    let mut names: Vec<String> = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Struct(data) if derives_component(&data.attrs) => {
                Some(data.ident.to_string())
            }
            _ => None,
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

// needed helper: `#[derive(Component)]` detection in any path form
fn derives_component(attrs: &[syn::Attribute]) -> bool {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .any(|attr| {
            attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .is_ok_and(|paths| {
                paths.iter().any(|path| {
                    path.segments
                        .last()
                        .is_some_and(|segment| segment.ident == "Component")
                })
            })
        })
}

#[cfg(test)]
mod tests {
    use super::declared_components;

    #[test]
    fn test_usage() {
        let file = syn::parse_str(
            "#[derive(Component)] pub struct Root;
             #[derive(Component, Debug)] pub struct Button;
             pub struct Plain;
             #[derive(Debug)] pub struct NotAComponent;",
        )
        .unwrap();
        let found = declared_components(&file);
        assert_eq!(found, vec!["Button".to_string(), "Root".to_string()]);
    }
}
