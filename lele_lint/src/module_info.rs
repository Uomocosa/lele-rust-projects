// needed helper: syn parsing utilities for mod.rs declarations
use std::path::{Path, PathBuf};

use crate::Entry;
use crate::ModDecl;
use crate::ModuleInfoMap;
use crate::Reexport;

use super::module_info_build;

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub rel_path: PathBuf,
    pub declarations: Vec<ModDecl>,
    pub reexports: Vec<Reexport>,
}

#[rustfmt::skip]
impl ModuleInfo {
    pub fn build(_src_dir: &Path, entries: &[Entry]) -> ModuleInfoMap {
        module_info_build::build(_src_dir, entries)
    }
}

#[cfg(test)]
mod tests {
    use crate::ModDecl;
    use crate::Reexport;

    #[test]
    fn test_usage() {
        let file = syn::parse_str::<syn::File>(
            "mod player;\nmod player_new;\npub mod bevy_systems;\npub use player::Player;\n",
        )
        .unwrap();

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
                        for tree_node in walk_tree(&u.tree) {
                            reexports.push(tree_node);
                        }
                    }
                }
                _ => {}
            }
        }

        assert_eq!(decls.len(), 3);
        assert_eq!(decls[0].name, "player");
        assert!(!decls[0].is_public);
        assert_eq!(decls[1].name, "player_new");
        assert!(!decls[1].is_public);
        assert_eq!(decls[2].name, "bevy_systems");
        assert!(decls[2].is_public);

        assert_eq!(reexports.len(), 1);
        assert_eq!(reexports[0].segments, vec!["player", "Player"]);
    }

    fn walk_tree(tree: &syn::UseTree) -> Vec<Reexport> {
        match tree {
            syn::UseTree::Path(p) => {
                let mut results = walk_tree(&p.tree);
                for r in &mut results {
                    r.segments.insert(0, p.ident.to_string());
                }
                results
            }
            syn::UseTree::Name(n) => vec![Reexport {
                segments: vec![n.ident.to_string()],
                is_glob: false,
            }],
            syn::UseTree::Glob(_) => vec![Reexport {
                segments: Vec::new(),
                is_glob: true,
            }],
            syn::UseTree::Rename(r) => vec![Reexport {
                segments: vec![r.ident.to_string()],
                is_glob: false,
            }],
            syn::UseTree::Group(_) => Vec::new(),
        }
    }
}
