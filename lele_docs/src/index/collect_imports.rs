use std::collections::HashMap;

use syn::Item;
use syn::UseTree;

use crate::index;

pub fn collect_imports(file: &syn::File) -> index::Imports {
    let mut map: HashMap<String, String> = HashMap::new();
    for item in &file.items {
        if let Item::Use(u) = item {
            collect_tree(&u.tree, &mut Vec::new(), &mut map);
        }
    }
    map
}

// needed helper: walk a use tree accumulating path segments
fn collect_tree(tree: &UseTree, prefix: &mut Vec<String>, map: &mut HashMap<String, String>) {
    match tree {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            collect_tree(&p.tree, prefix, map);
            prefix.pop();
        }
        UseTree::Name(n) => {
            let name = n.ident.to_string();
            let mut segs = prefix.clone();
            segs.push(name.clone());
            if let Some(canonical) = canonical(&segs) {
                map.insert(name, canonical);
            }
        }
        UseTree::Rename(r) => {
            let mut segs = prefix.clone();
            segs.push(r.ident.to_string());
            if let Some(canonical) = canonical(&segs) {
                map.insert(r.rename.to_string(), canonical);
            }
        }
        UseTree::Glob(_) => {}
        UseTree::Group(g) => {
            for item in &g.items {
                collect_tree(item, prefix, map);
            }
        }
    }
}

// needed helper: canonical crate-relative path (None for external imports)
fn canonical(segs: &[String]) -> Option<String> {
    let mut owned = segs.to_vec();
    if owned.first().map(String::as_str) == Some("crate") {
        owned.remove(0);
    } else {
        return None;
    }
    if owned.is_empty() {
        return None;
    }
    Some(owned.join("::"))
}

#[cfg(test)]
mod tests {
    use super::collect_imports;

    #[test]
    fn test_usage() {
        let file: syn::File =
            syn::parse_str("use crate::index;\nuse crate::frame::Frame;\nuse std::fmt;\n").unwrap();
        let imports = collect_imports(&file);
        assert_eq!(imports.get("index").map(String::as_str), Some("index"));
        assert_eq!(
            imports.get("Frame").map(String::as_str),
            Some("frame::Frame")
        );
        assert!(!imports.contains_key("fmt"));
    }
}
