use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use syn::visit::Visit;

use super::single_caller_type::SingleCallerType;
use crate::common;
use crate::diagnostic;
use crate::project;
use crate::severity;

pub(crate) fn check(
    _self: &SingleCallerType,
    project: &project::Project,
) -> Vec<diagnostic::Diagnostic> {
    let mut diags = Vec::new();
    let defined_types = collect_defined_types(&project.parsed_files);
    let defined_names: HashSet<String> = defined_types.iter().map(|(n, _)| n.clone()).collect();
    let embedded = collect_embedded_type_names(&project.parsed_files);
    let refs = collect_file_references(&project.parsed_files, &defined_names);

    for (name, rel_path) in &defined_types {
        if is_exempt_path(rel_path) {
            continue;
        }
        if embedded.contains(name) {
            continue;
        }
        let Some(file) = project.parsed_files.get(rel_path) else {
            continue;
        };
        if has_thin_delegate(file, name) {
            continue;
        }

        let callers: Vec<&PathBuf> = refs
            .iter()
            .filter(|(f, names)| *f != rel_path && names.contains(name))
            .map(|(f, _)| f)
            .collect();

        if callers.len() == 1 {
            diags.push(diagnostic::Diagnostic {
                file: project.src_dir.join(rel_path),
                line: 1,
                col: 0,
                code: "E016".to_string(),
                message: format!(
                    "type `{name}` has exactly one caller in `{}` and no thin-delegate methods — define it in the caller's file instead of its own file",
                    callers[0].display()
                ),
                severity: severity::Severity::Error,
            });
        }
    }

    diags
}

// needed helper: type definition collection across all files
fn collect_defined_types(parsed_files: &HashMap<PathBuf, syn::File>) -> Vec<(String, PathBuf)> {
    let mut types = Vec::new();
    for (rel_path, file) in parsed_files {
        for item in &file.items {
            let name = match item {
                syn::Item::Struct(s) => Some(s.ident.to_string()),
                syn::Item::Enum(e) => Some(e.ident.to_string()),
                _ => None,
            };
            if let Some(name) = name {
                types.push((name, rel_path.clone()));
            }
        }
    }
    types
}

// needed helper: path exemption for mod.rs/lib.rs
fn is_exempt_path(rel_path: &Path) -> bool {
    let file_name = rel_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    file_name == "mod.rs"
        || file_name == "lib.rs"
        || file_name == "constants.rs"
        || rel_path
            .components()
            .any(|c| c.as_os_str().to_str() == Some("tests"))
}

// needed helper: embedded type name collection from field types
fn collect_embedded_type_names(parsed_files: &HashMap<PathBuf, syn::File>) -> HashSet<String> {
    let mut names = HashSet::new();
    for file in parsed_files.values() {
        for item in &file.items {
            match item {
                syn::Item::Struct(s) => {
                    for field in &s.fields {
                        collect_type_paths(&field.ty, &mut names);
                    }
                }
                syn::Item::Enum(e) => {
                    for variant in &e.variants {
                        for field in variant.fields.iter() {
                            collect_type_paths(&field.ty, &mut names);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    names
}

// needed helper: type path visitor
fn collect_type_paths(ty: &syn::Type, names: &mut HashSet<String>) {
    struct Collect<'a>(&'a mut HashSet<String>);
    impl<'ast> syn::visit::Visit<'ast> for Collect<'_> {
        fn visit_path(&mut self, node: &'ast syn::Path) {
            if let Some(seg) = node.segments.last() {
                self.0.insert(seg.ident.to_string());
            }
            syn::visit::visit_path(self, node);
        }
    }
    Collect(names).visit_type(ty);
}

// needed helper: thin delegate method presence check
fn has_thin_delegate(file: &syn::File, type_name: &str) -> bool {
    file.items.iter().any(|item| {
        let syn::Item::Impl(impl_block) = item else {
            return false;
        };
        if common::self_type_last(&impl_block.self_ty).as_deref() != Some(type_name) {
            return false;
        }
        impl_is_all_delegate(impl_block)
    })
}

// needed helper: all-delegate impl block verification
fn impl_is_all_delegate(impl_block: &syn::ItemImpl) -> bool {
    let mut has_fn = false;
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            has_fn = true;
            if method.block.stmts.len() != 1 {
                return false;
            }
            if let syn::Stmt::Expr(syn::Expr::Call(call), _) = &method.block.stmts[0] {
                if let syn::Expr::Path(path) = call.func.as_ref() {
                    if path.path.segments.len() != 2 {
                        return false;
                    }
                    continue;
                }
            }
            return false;
        }
    }
    has_fn
}

// needed helper: per-file type reference collector
fn collect_file_references(
    parsed_files: &HashMap<PathBuf, syn::File>,
    defined: &HashSet<String>,
) -> HashMap<PathBuf, HashSet<String>> {
    let mut per_file = HashMap::new();
    for (rel_path, file) in parsed_files {
        let mut found = HashSet::new();
        collect_refs_from_items(&file.items, defined, &mut found);
        per_file.insert(rel_path.clone(), found);
    }
    per_file
}

// needed helper: item-level reference collection with cfg(test) skip
fn collect_refs_from_items(
    items: &[syn::Item],
    defined: &HashSet<String>,
    found: &mut HashSet<String>,
) {
    struct RefCollector<'a> {
        defined: &'a HashSet<String>,
        found: &'a mut HashSet<String>,
    }
    impl<'ast> syn::visit::Visit<'ast> for RefCollector<'_> {
        fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
            if common::is_cfg_test_mod(node) {
                return;
            }
            syn::visit::visit_item_mod(self, node);
        }

        fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
            if matches!(node.vis, syn::Visibility::Public(_)) {
                return;
            }
            syn::visit::visit_item_use(self, node);
        }

        fn visit_path(&mut self, node: &'ast syn::Path) {
            if let Some(seg) = node.segments.last() {
                if self.defined.contains(&seg.ident.to_string()) {
                    self.found.insert(seg.ident.to_string());
                }
            }
            syn::visit::visit_path(self, node);
        }
    }

    let mut collector = RefCollector { defined, found };
    for item in items {
        collector.visit_item(item);
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_defined_types, collect_embedded_type_names, has_thin_delegate};
    use std::collections::HashMap;

    #[test]
    fn test_usage_defined_types() {
        let mut map = HashMap::new();
        map.insert(
            std::path::PathBuf::from("player.rs"),
            syn::parse_str("pub struct Player { pub x: u32 }\n").unwrap(),
        );
        let types = collect_defined_types(&map);
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].0, "Player");
    }

    #[test]
    fn test_usage_enum_defined() {
        let mut map = HashMap::new();
        map.insert(
            std::path::PathBuf::from("msg.rs"),
            syn::parse_str("pub enum Msg { Ping, Quit }\n").unwrap(),
        );
        let types = collect_defined_types(&map);
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].0, "Msg");
    }

    #[test]
    fn test_usage_embedded() {
        let mut map = HashMap::new();
        map.insert(
            std::path::PathBuf::from("config.rs"),
            syn::parse_str("pub struct Config { pub sec: Option<LeleLintSection> }\n").unwrap(),
        );
        let embedded = collect_embedded_type_names(&map);
        assert!(embedded.contains("Option"));
        assert!(embedded.contains("LeleLintSection"));
    }

    #[test]
    fn test_usage_has_thin_delegate() {
        let file: syn::File = syn::parse_str(
            "#[rustfmt::skip] impl Foo { pub fn new() -> Self { config_new::new() } }",
        )
        .unwrap();
        assert!(has_thin_delegate(&file, "Foo"));
    }

    #[test]
    fn test_usage_no_thin_delegate() {
        let file: syn::File =
            syn::parse_str("impl Foo { pub fn new() -> Self { Self { x: 1 } } }").unwrap();
        assert!(!has_thin_delegate(&file, "Foo"));
    }
}

// no test_usage necessary
