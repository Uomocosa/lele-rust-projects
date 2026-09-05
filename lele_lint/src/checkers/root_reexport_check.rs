use std::collections::HashSet;
use std::path::PathBuf;

use super::root_reexport::RootReexport;
use crate::common;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &RootReexport, project: &Project) -> Vec<Diagnostic> {
    let lib = match project.parsed_files.get(&PathBuf::from("lib.rs")) {
        Some(lib) => lib,
        None => return Vec::new(),
    };
    let pub_mods = public_modules(lib);
    let reexported = root_reexports(lib);
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        if rel_path.components().count() != 1 {
            continue;
        }
        let stem = rel_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if matches!(stem, "lib" | "main" | "mod") {
            continue;
        }
        if !pub_mods.contains(stem) {
            continue;
        }
        for missing in missing_reexports(stem, file, &reexported) {
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line: missing.line,
                col: 0,
                code: RootReexport::CODE.to_string(),
                message: format!(
                    "public type `{}` in root module `{stem}` is not re-exported at the crate root — add `pub use {stem}::{};` to lib.rs",
                    missing.ty, missing.ty
                ),
                severity: Severity::Error,
            });
        }
        for missing in missing_fn_flatten(stem, file, lib) {
            diags.push(Diagnostic {
                file: project.src_dir.join("lib.rs"),
                line: 1,
                col: 0,
                code: RootReexport::CODE.to_string(),
                message: missing,
                severity: Severity::Error,
            });
        }
    }

    diags
}

// needed helper: SHAPE-F fn flatten — stutter fn-file `stem.rs` holding `pub fn stem`
// requires private `mod stem;` plus `pub use stem::stem;` in lib.rs
fn missing_fn_flatten(stem: &str, file: &syn::File, lib: &syn::File) -> Vec<String> {
    let mut fns = Vec::new();
    for item in &file.items {
        if let syn::Item::Fn(f) = item {
            if !matches!(f.vis, syn::Visibility::Public(_)) {
                continue;
            }
            if common::to_snake_case(&f.sig.ident.to_string()) == stem {
                fns.push(f.sig.ident.to_string());
            }
        }
    }
    if fns.is_empty() {
        return Vec::new();
    }
    let has_private_mod = lib.items.iter().any(|item| {
        matches!(item, syn::Item::Mod(m) if m.ident == stem && matches!(m.vis, syn::Visibility::Inherited))
    });
    let reexported: HashSet<(String, String)> = root_reexports(lib);
    let mut missing = Vec::new();
    for name in fns {
        if !has_private_mod {
            missing.push(format!(
                "SHAPE-F fn-file `{stem}.rs` holds `pub fn {name}` — declare private `mod {stem};` in lib.rs (not `pub mod`)"
            ));
        }
        if !reexported.contains(&(stem.to_string(), name.clone())) {
            missing.push(format!(
                "SHAPE-F fn-file `{stem}.rs` holds `pub fn {name}` — add `pub use {stem}::{name};` to lib.rs for `crate::{name}()`"
            ));
        }
    }
    missing
}

// needed helper: missing re-export detection for one root file
struct MissingReexport {
    line: usize,
    ty: String,
}

fn missing_reexports(
    stem: &str,
    file: &syn::File,
    reexported: &HashSet<(String, String)>,
) -> Vec<MissingReexport> {
    let mut missing = Vec::new();
    for item in &file.items {
        let (name, span) = match item {
            syn::Item::Struct(s) if matches!(s.vis, syn::Visibility::Public(_)) => {
                (s.ident.to_string(), s.ident.span())
            }
            syn::Item::Enum(e) if matches!(e.vis, syn::Visibility::Public(_)) => {
                (e.ident.to_string(), e.ident.span())
            }
            syn::Item::Type(t) if matches!(t.vis, syn::Visibility::Public(_)) => {
                (t.ident.to_string(), t.ident.span())
            }
            syn::Item::Trait(t) if matches!(t.vis, syn::Visibility::Public(_)) => {
                (t.ident.to_string(), t.ident.span())
            }
            _ => continue,
        };
        if !common::is_stuttered_path(stem, &name) {
            continue;
        }
        if !reexported.contains(&(stem.to_string(), name.clone())) {
            missing.push(MissingReexport {
                line: span.start().line,
                ty: name,
            });
        }
    }
    missing
}

// needed helper: public module declarations in lib.rs
fn public_modules(lib: &syn::File) -> HashSet<String> {
    lib.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Mod(m) if matches!(m.vis, syn::Visibility::Public(_)) => {
                Some(m.ident.to_string())
            }
            _ => None,
        })
        .collect()
}

// needed helper: exact `pub use module::Type;` pairs in lib.rs
fn root_reexports(lib: &syn::File) -> HashSet<(String, String)> {
    let mut pairs = HashSet::new();
    for item in &lib.items {
        if let syn::Item::Use(u) = item {
            if !matches!(u.vis, syn::Visibility::Public(_)) {
                continue;
            }
            if let syn::UseTree::Path(path) = &u.tree {
                if let syn::UseTree::Name(name) = path.tree.as_ref() {
                    pairs.insert((path.ident.to_string(), name.ident.to_string()));
                }
            }
        }
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::{missing_fn_flatten, missing_reexports, public_modules, root_reexports};

    fn parse(src: &str) -> syn::File {
        syn::parse_str(src).unwrap()
    }

    #[test]
    fn test_usage_flags_missing_reexport() {
        let lib = parse("pub mod diagnostic;\n");
        let file = parse("pub struct Diagnostic { pub x: u32 }\n");
        let missing = missing_reexports("diagnostic", &file, &root_reexports(&lib));
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].ty, "Diagnostic");
        assert!(public_modules(&lib).contains("diagnostic"));
    }

    #[test]
    fn test_usage_allows_reexported_type() {
        let lib = parse("pub mod diagnostic;\npub use diagnostic::Diagnostic;\n");
        let file = parse("pub struct Diagnostic { pub x: u32 }\n");
        let missing = missing_reexports("diagnostic", &file, &root_reexports(&lib));
        assert!(missing.is_empty());
    }

    #[test]
    fn test_usage_allows_function_modules() {
        let lib = parse("pub mod discover;\n");
        let file = parse("pub fn discover() {}\n");
        let missing = missing_reexports("discover", &file, &root_reexports(&lib));
        assert!(missing.is_empty());
    }

    #[test]
    fn test_usage_allows_non_stutter_types() {
        let lib = parse("pub mod module_info;\n");
        let file = parse("pub type ModuleInfoMap = u32;\n");
        let missing = missing_reexports("module_info", &file, &root_reexports(&lib));
        assert!(missing.is_empty());
    }

    #[test]
    fn test_usage_ignores_private_modules() {
        let lib = parse("mod config_load;\n");
        assert!(!public_modules(&lib).contains("config_load"));
    }

    #[test]
    fn test_usage_flags_missing_fn_flatten() {
        let lib = parse("pub mod discover;\n");
        let file = parse("pub fn discover() {}\n");
        let missing = missing_fn_flatten("discover", &file, &lib);
        assert_eq!(missing.len(), 2);
        assert!(missing[0].contains("private `mod discover;`"));
        assert!(missing[1].contains("pub use discover::discover;"));
    }

    #[test]
    fn test_usage_allows_flat_fn() {
        let lib = parse("mod discover;\npub use discover::discover;\n");
        let file = parse("pub fn discover() {}\n");
        assert_eq!(missing_fn_flatten("discover", &file, &lib).len(), 0);
    }

    #[test]
    fn test_usage_ignores_non_stutter_fn() {
        let lib = parse("pub mod discover;\n");
        let file = parse("pub fn run() {}\n");
        assert_eq!(missing_fn_flatten("discover", &file, &lib).len(), 0);
    }
}
