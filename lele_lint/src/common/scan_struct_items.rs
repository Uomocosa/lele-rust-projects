use std::path::Path;

use crate::Diagnostic;

#[must_use]
pub(crate) fn scan_struct_items(
    items: &[syn::Item],
    file_path: &Path,
    check: impl Fn(&syn::ItemStruct, &Path, &mut Vec<Diagnostic>),
) -> Vec<Diagnostic> {
    fn recursion(
        items: &[syn::Item],
        file_path: &Path,
        check: &impl Fn(&syn::ItemStruct, &Path, &mut Vec<Diagnostic>),
        diags: &mut Vec<Diagnostic>,
    ) {
        for item in items {
            match item {
                syn::Item::Struct(struct_def) => check(struct_def, file_path, diags),
                syn::Item::Mod(module) => {
                    if let Some((_, inner)) = &module.content {
                        recursion(inner, file_path, check, diags);
                    }
                }
                _ => {}
            }
        }
    }

    let mut diags = Vec::new();
    recursion(items, file_path, &check, &mut diags);
    diags
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::Diagnostic;

    use super::scan_struct_items;

    #[test]
    fn test_usage() {
        let file = syn::parse_str::<syn::File>("mod inner { pub struct A(pub u64); }").unwrap();
        let found = scan_struct_items(&file.items, Path::new("x.rs"), |_, _, diags| {
            diags.push(Diagnostic {
                file: std::path::PathBuf::from("x.rs"),
                line: 1,
                col: 0,
                code: "E018",
                message: String::new(),
            });
        });
        assert_eq!(found.len(), 1);
    }
}
