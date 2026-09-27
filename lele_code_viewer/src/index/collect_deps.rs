use std::collections::BTreeSet;

use syn::Item;
use syn::UseTree;

use crate::index;

pub fn collect_deps(file: &syn::File) -> index::FileDeps {
    let mut internal: BTreeSet<String> = BTreeSet::new();
    let mut external: BTreeSet<String> = BTreeSet::new();
    for item in &file.items {
        if let Item::Use(u) = item {
            collect_tree(&u.tree, &mut Vec::new(), &mut internal, &mut external);
        }
    }
    index::FileDeps {
        internal: internal.into_iter().collect(),
        external: external.into_iter().collect(),
    }
}

// needed helper: walk a use tree recording internal module paths and external crates
fn collect_tree(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    internal: &mut BTreeSet<String>,
    external: &mut BTreeSet<String>,
) {
    match tree {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            collect_tree(&p.tree, prefix, internal, external);
            prefix.pop();
        }
        UseTree::Name(n) => record(prefix, &n.ident.to_string(), internal, external),
        UseTree::Rename(r) => record(prefix, &r.ident.to_string(), internal, external),
        UseTree::Glob(_) => record(prefix, "", internal, external),
        UseTree::Group(g) => {
            for item in &g.items {
                collect_tree(item, prefix, internal, external);
            }
        }
    }
}

// needed helper: classify one fully-qualified import path
fn record(
    prefix: &[String],
    name: &str,
    internal: &mut BTreeSet<String>,
    external: &mut BTreeSet<String>,
) {
    let Some(first) = prefix.first().map(String::as_str) else {
        return;
    };
    if matches!(first, "self" | "super" | "std" | "core" | "alloc") {
        if matches!(first, "self" | "super") {
            return;
        }
        external.insert(first.to_string());
        return;
    }
    if first == "crate" {
        let mut segs: Vec<String> = prefix.iter().skip(1).cloned().collect();
        if !name.is_empty() {
            segs.push(name.to_string());
        }
        if !segs.is_empty() {
            internal.insert(segs.join("::"));
        }
        return;
    }
    external.insert(first.to_string());
}

#[cfg(test)]
mod tests {
    use super::collect_deps;

    #[test]
    fn test_usage() {
        let file: syn::File = syn::parse_str(
            "use crate::index::build_index;\nuse crate::render;\nuse std::fmt;\nuse axum::Router;\n",
        )
        .unwrap();
        let deps = collect_deps(&file);
        assert!(deps.internal.contains(&"index::build_index".to_string()));
        assert!(deps.internal.contains(&"render".to_string()));
        assert!(deps.external.contains(&"std".to_string()));
        assert!(deps.external.contains(&"axum".to_string()));
    }
}
