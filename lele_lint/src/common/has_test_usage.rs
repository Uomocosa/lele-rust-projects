use crate::common;

pub(crate) fn has_test_usage(file: &syn::File) -> bool {
    file.items.iter().any(|item| {
        let syn::Item::Mod(module) = item else {
            return false;
        };
        if !common::is_cfg_test_mod(module) {
            return false;
        }
        module.content.as_ref().is_some_and(|(_, items)| {
            items.iter().any(|inner| {
                matches!(
                    inner,
                    syn::Item::Fn(func) if func.sig.ident == "test_usage"
                )
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::has_test_usage;

    #[test]
    fn test_usage() {
        let with: syn::File =
            syn::parse_str("#[cfg(test)] mod tests { #[test] fn test_usage() {} }").unwrap();
        assert!(has_test_usage(&with));

        let without: syn::File = syn::parse_str("pub fn compute(x: u32) -> u32 { x * 2 }").unwrap();
        assert!(!has_test_usage(&without));
    }
}
