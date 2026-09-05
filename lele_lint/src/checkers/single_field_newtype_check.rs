use std::path::Path;

use super::single_field_newtype::SingleFieldNewtype;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &SingleFieldNewtype, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        scan_items(&file.items, rel_path, project, &mut diags);
    }

    diags
}

// needed helper: recursive item scanner
fn scan_items(
    items: &[syn::Item],
    rel_path: &Path,
    project: &Project,
    diags: &mut Vec<Diagnostic>,
) {
    for item in items {
        match item {
            syn::Item::Struct(struct_def) => check_struct(struct_def, rel_path, project, diags),
            syn::Item::Mod(module) => {
                if let Some((_, inner)) = &module.content {
                    scan_items(inner, rel_path, project, diags);
                }
            }
            _ => {}
        }
    }
}

// needed helper: single-field struct shape validation
fn check_struct(
    struct_def: &syn::ItemStruct,
    rel_path: &Path,
    project: &Project,
    diags: &mut Vec<Diagnostic>,
) {
    let name = struct_def.ident.to_string();
    let field_count = struct_def.fields.len();

    if field_count == 1 {
        if let syn::Fields::Named(_) = struct_def.fields {
            if has_data_shape_derive(struct_def) {
                return;
            }
            if has_deref_derive(struct_def) {
                return;
            }
            push(diags, rel_path, project, format!("{name} has a single named field without Deref and must be a tuple newtype like `pub struct {name}(pub T)`; named single-field structs must derive Deref"));
            return;
        }
        if !has_deref_derive(struct_def) {
            push(
                diags,
                rel_path,
                project,
                format!("{name} is a single-field tuple newtype and must derive Deref"),
            );
        }
    } else if field_count >= 2 {
        if let syn::Fields::Unnamed(_) = struct_def.fields {
            push(
                diags,
                rel_path,
                project,
                format!(
                    "{name} has multiple fields and must use named fields like `{{ a: A, b: B }}`"
                ),
            );
        }
    }
}

// needed helper: derive Deref attribute presence check
fn has_deref_derive(struct_def: &syn::ItemStruct) -> bool {
    has_derive_name(struct_def, &["Deref"])
}

// needed helper: wire-shape derive exemption (serde/clap field names come from the format)
fn has_data_shape_derive(struct_def: &syn::ItemStruct) -> bool {
    has_derive_name(
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

// needed helper: derive list name matching (bare and path-suffixed)
fn has_derive_name(struct_def: &syn::ItemStruct, names: &[&str]) -> bool {
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

// needed helper: diagnostic emission
fn push(diags: &mut Vec<Diagnostic>, rel_path: &Path, project: &Project, message: String) {
    diags.push(Diagnostic {
        file: project.src_dir.join(rel_path),
        line: 1,
        col: 0,
        code: "E018".to_string(),
        message,
        severity: Severity::Error,
    });
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::Project;

    use super::check_struct;
    use super::has_data_shape_derive;
    use super::has_deref_derive;
    use syn::ItemStruct;

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
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("tuple newtype"));
    }

    #[test]
    fn test_usage_single_named_with_deref_passes() {
        let s = parse("#[derive(Debug, Clone, Deref)] pub struct X { pub value: Vec<String> }");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_single_field_without_deref_is_rejected() {
        let s = parse("pub struct X(pub u64);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("derive Deref"));
    }

    #[test]
    fn test_usage_single_field_with_deref_passes() {
        let s = parse("#[derive(Deref)] pub struct X(pub u64);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
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
            assert!(has_data_shape_derive(&s));
            let mut diags = Vec::new();
            check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
            assert!(diags.is_empty());
        }
    }

    #[test]
    fn test_usage_non_shape_single_field_rejected() {
        let s = parse("#[derive(Clone, Debug)] struct X { value: u64 }");
        assert!(!has_data_shape_derive(&s));
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_usage_multi_field_tuple_is_rejected() {
        let s = parse("pub struct X(pub String, pub u32);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("named fields"));
    }

    fn default_project() -> Project {
        Project {
            root: PathBuf::from("."),
            src_dir: PathBuf::from("src"),
            entries: Vec::new(),
            module_info: std::collections::HashMap::default(),
            parsed_files: std::collections::HashMap::default(),
        }
    }
}

// no test_usage necessary
