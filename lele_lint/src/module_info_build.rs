// no test_usage necessary
// needed helper: mod_rs parsing utilities
use std::path::Path;

use super::module_info::{ModuleInfo, ModuleInfoMap};
use crate::entry::Entry;
use crate::mod_decl::ModDecl;
use crate::reexport::Reexport;

pub fn build(_src_dir: &Path, entries: &[Entry]) -> ModuleInfoMap {
    let mut map = ModuleInfoMap::new();

    for entry in entries {
        if !is_mod_rs(&entry.relative_path) {
            continue;
        }

        let content = match std::fs::read_to_string(&entry.absolute_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (declarations, reexports) = parse_mod_rs(&content);

        map.insert(
            entry.relative_path.clone(),
            ModuleInfo {
                rel_path: entry.relative_path.clone(),
                declarations,
                reexports,
            },
        );
    }

    map
}

fn is_mod_rs(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n == "mod.rs")
}

fn parse_mod_rs(content: &str) -> (Vec<ModDecl>, Vec<Reexport>) {
    let file = match syn::parse_file(content) {
        Ok(f) => f,
        Err(_) => return (Vec::new(), Vec::new()),
    };

    let mut decls = Vec::new();
    let mut reexports = Vec::new();

    for item in file.items {
        match item {
            syn::Item::Mod(m) => {
                decls.push(ModDecl {
                    name: m.ident.to_string(),
                    is_public: matches!(m.vis, syn::Visibility::Public(_)),
                });
            }
            syn::Item::Use(u) => {
                if matches!(u.vis, syn::Visibility::Public(_)) {
                    if let Some(r) = extract_reexport(&u.tree) {
                        reexports.push(r);
                    }
                }
            }
            _ => {}
        }
    }

    (decls, reexports)
}

fn extract_reexport(tree: &syn::UseTree) -> Option<Reexport> {
    match tree {
        syn::UseTree::Path(p) => {
            let mut segments = vec![p.ident.to_string()];
            if let Some(child) = extract_reexport(&p.tree) {
                segments.extend(child.segments);
                Some(Reexport {
                    segments,
                    is_glob: child.is_glob,
                })
            } else {
                None
            }
        }
        syn::UseTree::Name(n) => Some(Reexport {
            segments: vec![n.ident.to_string()],
            is_glob: false,
        }),
        syn::UseTree::Glob(_) => Some(Reexport {
            segments: Vec::new(),
            is_glob: true,
        }),
        syn::UseTree::Rename(r) => Some(Reexport {
            segments: vec![r.ident.to_string()],
            is_glob: false,
        }),
        syn::UseTree::Group(_) => None,
    }
}
