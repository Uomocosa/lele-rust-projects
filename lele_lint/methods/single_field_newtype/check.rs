use std::path::Path;

use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::Project;

pub fn check(
    _self: &checkers::single_field_newtype::SingleFieldNewtype,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for source in project.content_sources() {
        let file_path = project.absolute_path(source.origin, source.relative_path);
        scan_items(&source.file.items, &file_path, &mut diags);
    }

    diags
}

// needed helper: recursive item scanner
fn scan_items(items: &[syn::Item], file_path: &Path, diags: &mut Vec<Diagnostic>) {
    for item in items {
        match item {
            syn::Item::Struct(struct_def) => check_struct(struct_def, file_path, diags),
            syn::Item::Mod(module) => {
                if let Some((_, inner)) = &module.content {
                    scan_items(inner, file_path, diags);
                }
            }
            _ => {}
        }
    }
}

// needed helper: single-field struct shape validation
fn check_struct(struct_def: &syn::ItemStruct, file_path: &Path, diags: &mut Vec<Diagnostic>) {
    let name = struct_def.ident.to_string();
    let field_count = struct_def.fields.len();

    if field_count == 1 {
        if let syn::Fields::Named(_) = struct_def.fields {
            if common::has_data_shape_derive(struct_def) {
                return;
            }
            if has_deref_derive(struct_def) {
                return;
            }
            push(diags, file_path, format!("{name} has a single named field without Deref and must be a tuple newtype like `pub struct {name}(pub T)`; named single-field structs must derive Deref"));
            return;
        }
        if !has_deref_derive(struct_def) {
            push(
                diags,
                file_path,
                format!("{name} is a single-field tuple newtype and must derive Deref"),
            );
        }
    } else if field_count >= 2 {
        if let syn::Fields::Unnamed(_) = struct_def.fields {
            push(
                diags,
                file_path,
                format!(
                    "{name} has multiple fields and must use named fields like `{{ a: A, b: B }}`"
                ),
            );
        }
    }
}

// needed helper: derive Deref attribute presence check
fn has_deref_derive(struct_def: &syn::ItemStruct) -> bool {
    common::has_derive_name(struct_def, &["Deref"])
}

// needed helper: diagnostic emission
fn push(diags: &mut Vec<Diagnostic>, file_path: &Path, message: String) {
    diags.push(Diagnostic {
        file: file_path.to_path_buf(),
        line: 1,
        col: 0,
        code: checkers::single_field_newtype::SingleFieldNewtype::CODE.to_string(),
        message,
    });
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::Project;

    use super::check;
    use super::check_struct;
    use super::has_deref_derive;
    use crate::checkers;
    use crate::common;
    use syn::ItemStruct;

    #[test]
    fn test_usage() {
        let checker = checkers::single_field_newtype::SingleFieldNewtype;
        let project = Project::default();
        assert!(check(&checker, &project).is_empty());
    }

    fn parse(code: &str) -> ItemStruct {
        syn::parse_str(code).unwrap()
    }

    #[test]
    fn test_usage_detects_deref_derive() {
        assert!(has_deref_derive(&parse(
            "#[derive(Deref)] pub struct X(pub u64);"
        )));
        assert!(has_deref_derive(&parse(
            "#[derive(Clone, Deref, DerefMut)] pub struct X(pub u64);"
        )));
        assert!(!has_deref_derive(&parse(
            "#[derive(Clone)] pub struct X(pub u64);"
        )));
        assert!(!has_deref_derive(&parse("pub struct X(pub u64);")));
    }

    #[test]
    fn test_usage_single_field_named_is_rejected() {
        let s = parse("pub struct X { pub value: u64 }");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("tuple newtype"));
    }

    #[test]
    fn test_usage_single_named_with_deref_passes() {
        let s = parse("#[derive(Debug, Clone, Deref)] pub struct X { pub value: Vec<String> }");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_single_field_without_deref_is_rejected() {
        let s = parse("pub struct X(pub u64);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("derive Deref"));
    }

    #[test]
    fn test_usage_single_field_with_deref_passes() {
        let s = parse("#[derive(Deref)] pub struct X(pub u64);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_serde_single_field_passes() {
        for code in [
            "#[derive(Deserialize)] struct Cfg { lele: String }",
            "#[derive(Serialize, Deserialize)] struct Cfg { lele: String }",
            "#[derive(serde::Deserialize)] struct Cfg { lele: String }",
            "#[derive(Parser)] struct Args { root: String }",
            "#[derive(Args)] struct Cmd { root: String }",
        ] {
            let s = parse(code);
            assert!(common::has_data_shape_derive(&s));
            let mut diags = Vec::new();
            check_struct(&s, Path::new("x.rs"), &mut diags);
            assert!(diags.is_empty());
        }
    }

    #[test]
    fn test_usage_non_shape_single_field_rejected() {
        let s = parse("#[derive(Clone, Debug)] struct X { value: u64 }");
        assert!(!common::has_data_shape_derive(&s));
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_usage_multi_field_tuple_is_rejected() {
        let s = parse("pub struct X(pub String, pub u32);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("named fields"));
    }
}

// no test_usage necessary
