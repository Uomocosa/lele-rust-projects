use std::path::Path;

use crate::Entry;
use crate::ModDecl;
use crate::ModuleInfo;
use crate::ModuleInfoMap;
use crate::Reexport;

pub fn build(_src_dir: &Path, entries: &[Entry]) -> ModuleInfoMap {
    let mut map = ModuleInfoMap::new();

    for entry in entries {
        if !is_index_file(&entry.relative_path) {
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

// needed helper: crate root or directory index filename check
fn is_index_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| matches!(n, "mod.rs" | "lib.rs" | "main.rs"))
}

// needed helper: mod.rs AST parsing for declarations and re-exports
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
                    cfg: cfg_attribute(&m.attrs),
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

// needed helper: stringify the first #[cfg(...)] predicate on a module declaration
fn cfg_attribute(attrs: &[syn::Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("cfg") {
            return None;
        }
        let meta = attr.parse_args::<syn::Meta>().ok()?;
        Some(quote::quote!(#meta).to_string())
    })
}

// needed helper: re-export path extraction from use tree
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::build;
    use crate::Entry;
    use crate::EntryKind;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let rel = PathBuf::from("clicker/mod.rs");
        let abs = dir.path().join("mod.rs");
        std::fs::write(&abs, "mod foo;\npub use foo::Foo;\n").unwrap();
        let entries = vec![Entry {
            relative_path: rel.clone(),
            absolute_path: abs,
            kind: EntryKind::File,
        }];
        let map = build(dir.path(), &entries);
        let info = map.get(&rel).unwrap();
        assert_eq!(info.declarations.len(), 1);
        assert_eq!(info.declarations[0].name, "foo");
        assert_eq!(info.reexports.len(), 1);
        assert_eq!(info.reexports[0].segments, vec!["foo", "Foo"]);
    }
}
