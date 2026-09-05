use std::path::Path;

use super::no_collection_newtype::NoCollectionNewtype;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &NoCollectionNewtype, project: &Project) -> Vec<Diagnostic> {
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

// needed helper: single-field collection inner-type validation
fn check_struct(
    struct_def: &syn::ItemStruct,
    rel_path: &Path,
    project: &Project,
    diags: &mut Vec<Diagnostic>,
) {
    if struct_def.fields.len() != 1 {
        return;
    }
    let syn::Fields::Unnamed(fields) = &struct_def.fields else {
        return;
    };
    let Some(field) = fields.unnamed.first() else {
        return;
    };
    if has_data_shape_derive(struct_def) {
        return;
    }
    let Some(collection) = collection_name(&field.ty) else {
        return;
    };
    if has_unsigned_element(&field.ty) {
        return;
    }
    let name = struct_def.ident.to_string();
    let singular = singular_name(&name);
    diags.push(Diagnostic {
        file: project.src_dir.join(rel_path),
        line: 1,
        col: 0,
        code: "E028".to_string(),
        message: format!(
            "{name} wraps {collection}<T>; define singular {singular}(T) with Deref and use Vec<{singular}> at call sites"
        ),
        severity: Severity::Error,
    });
}

// needed helper: collection path-segment matching (bare and path-suffixed)
fn collection_name(ty: &syn::Type) -> Option<&'static str> {
    if let syn::Type::Reference(ref_ty) = ty {
        if matches!(*ref_ty.elem, syn::Type::Slice(_)) {
            if ref_ty.mutability.is_some() {
                return Some("&mut [T]");
            }
            return Some("&[T]");
        }
        return None;
    }
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    let name = segment.ident.to_string();
    match name.as_str() {
        "Vec" => Some("Vec"),
        "HashSet" => Some("HashSet"),
        "BTreeSet" => Some("BTreeSet"),
        "VecDeque" => Some("VecDeque"),
        "HashMap" => Some("HashMap"),
        "BTreeMap" => Some("BTreeMap"),
        "Box" => {
            if boxed_slice_element(segment).is_some() {
                Some("Box<[T]>")
            } else {
                None
            }
        }
        // Rc<[T]> and Arc<[T]> are currently accepted; needs further investigation
        // whether shared-ownership slices follow the same singular-newtype rule.
        _ => None,
    }
}

// needed helper: Box<[T]> inner slice element extraction
fn boxed_slice_element(segment: &syn::PathSegment) -> Option<&syn::Type> {
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    for arg in &args.args {
        if let syn::GenericArgument::Type(syn::Type::Slice(slice)) = arg {
            return Some(&slice.elem);
        }
    }
    None
}

// needed helper: unsigned-int element exemption for byte-buffer-like newtypes
fn has_unsigned_element(ty: &syn::Type) -> bool {
    element_type(ty).is_some_and(is_unsigned_int)
}

// needed helper: first element type extraction across Vec-like and slice-like shapes
fn element_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Reference(ref_ty) = ty {
        if let syn::Type::Slice(slice) = &*ref_ty.elem {
            return Some(&slice.elem);
        }
        return None;
    }
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    let name = segment.ident.to_string();
    match name.as_str() {
        "Box" => boxed_slice_element(segment),
        "HashMap" | "BTreeMap" => None,
        _ => first_generic_type(segment),
    }
}

// needed helper: first generic type argument extraction
fn first_generic_type(segment: &syn::PathSegment) -> Option<&syn::Type> {
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    for arg in &args.args {
        if let syn::GenericArgument::Type(inner) = arg {
            return Some(inner);
        }
    }
    None
}

// needed helper: unsigned-int ident matching for uX exemption
fn is_unsigned_int(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    if !matches!(
        segment.ident.to_string().as_str(),
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
    ) {
        return false;
    }
    matches!(segment.arguments, syn::PathArguments::None)
}

// needed helper: wire-shape derive exemption (matches E018)
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

// needed helper: naive plural-to-singular suggestion for the message
fn singular_name(name: &str) -> String {
    if let Some(stripped) = name.strip_suffix("ies") {
        format!("{stripped}y")
    } else if let Some(stripped) = name.strip_suffix("ses") {
        stripped.to_string()
    } else if let Some(stripped) = name.strip_suffix("s") {
        stripped.to_string()
    } else {
        format!("{name}Item")
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::Project;

    use super::check_struct;
    use super::singular_name;
    use syn::ItemStruct;

    fn parse(code: &str) -> ItemStruct {
        syn::parse_str(code).unwrap()
    }

    #[test]
    fn test_usage() {
        let s = parse("pub struct ExampleNames(pub Vec<String>);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("Vec<ExampleName>"));
    }

    #[test]
    fn test_usage_singular_newtype_passes() {
        let s = parse("pub struct ExampleName(pub String);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_map_newtype_is_rejected() {
        let s = parse("pub struct Scores(pub HashMap<String, u32>);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_usage_multi_field_vec_passes() {
        let s = parse("pub struct Player { pub names: Vec<String> }");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_serde_exempt() {
        let s = parse("#[derive(Deserialize)] pub struct Tags(pub Vec<String>);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_usage_singular_names() {
        assert_eq!(singular_name("ExampleNames"), "ExampleName");
        assert_eq!(singular_name("Tags"), "Tag");
        assert_eq!(singular_name("Entries"), "Entry");
        assert_eq!(singular_name("Score"), "ScoreItem");
    }

    #[test]
    fn test_usage_borrowed_slice_rejected() {
        let s = parse("pub struct ExampleNames<'a>(pub &'a [String]);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_usage_mut_slice_rejected() {
        let s = parse("pub struct ExampleNames<'a>(pub &'a mut [String]);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_usage_boxed_slice_rejected() {
        let s = parse("pub struct ExampleNames(pub Box<[String]>);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_usage_unsigned_elements_exempt() {
        for code in [
            "pub struct Bytes(pub Vec<u8>);",
            "pub struct Counts(pub Vec<usize>);",
            "pub struct Words(pub Vec<u32>);",
        ] {
            let s = parse(code);
            let mut diags = Vec::new();
            check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
            assert!(diags.is_empty());
        }
    }

    #[test]
    fn test_usage_unsigned_slice_exempt() {
        let s = parse("pub struct Bytes<'a>(pub &'a [u8]);");
        let mut diags = Vec::new();
        check_struct(&s, Path::new("x.rs"), &default_project(), &mut diags);
        assert!(diags.is_empty());
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
