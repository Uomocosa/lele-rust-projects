use std::path::Path;

use super::single_field_newtype::SingleFieldNewtype;
use crate::diagnostic;
use crate::project;
use crate::severity;

pub(crate) fn check(
    _self: &SingleFieldNewtype,
    project: &project::Project,
) -> Vec<diagnostic::Diagnostic> {
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
    project: &project::Project,
    diags: &mut Vec<diagnostic::Diagnostic>,
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
    project: &project::Project,
    diags: &mut Vec<diagnostic::Diagnostic>,
) {
    let name = struct_def.ident.to_string();
    let field_count = struct_def.fields.len();

    if field_count == 1 {
        if let syn::Fields::Named(_) = struct_def.fields {
            push(diags, rel_path, project, format!("{name} has a single field and must be a tuple newtype like `pub struct {name}(pub T)`"));
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
    struct_def.attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        let syn::Meta::List(list) = &attr.meta else {
            return false;
        };
        list.tokens
            .to_string()
            .split(',')
            .any(|t| t.trim() == "Deref")
    })
}

// needed helper: diagnostic emission
fn push(
    diags: &mut Vec<diagnostic::Diagnostic>,
    rel_path: &Path,
    project: &project::Project,
    message: String,
) {
    diags.push(diagnostic::Diagnostic {
        file: project.src_dir.join(rel_path),
        line: 1,
        col: 0,
        code: "E018".to_string(),
        message,
        severity: severity::Severity::Error,
    });
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::project;

    use super::check_struct;
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
    fn test_usage_multi_field_tuple_is_rejected() {
        let s = parse("pub struct X(pub String, pub u32);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("named fields"));
    }

    fn default_project() -> project::Project {
        project::Project {
            root: PathBuf::from("."),
            src_dir: PathBuf::from("src"),
            entries: Vec::new(),
            module_info: Default::default(),
            parsed_files: Default::default(),
        }
    }
}

// no test_usage necessary
