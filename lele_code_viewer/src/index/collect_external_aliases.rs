use syn::Item;
use syn::UseTree;

use crate::index;

pub fn collect_external_aliases(file: &syn::File) -> index::ExternalAliases {
    let mut out = index::ExternalAliases::default();
    for item in &file.items {
        if let Item::Use(node) = item {
            collect_tree(&node.tree, &mut Vec::new(), &mut out);
        }
    }
    out
}

// needed helper: walk a use tree mapping each imported name to its root crate
fn collect_tree(tree: &UseTree, prefix: &mut Vec<String>, out: &mut index::ExternalAliases) {
    match tree {
        UseTree::Path(node) => {
            prefix.push(node.ident.to_string());
            collect_tree(&node.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(node) => record(prefix, &node.ident.to_string(), out),
        UseTree::Rename(node) => {
            if let Some(root) = prefix.first()
                && !matches!(root.as_str(), "self" | "super" | "crate")
            {
                out.local_to_crate
                    .insert(node.rename.to_string(), root.clone());
            }
        }
        UseTree::Glob(_) => {
            if let Some(root) = prefix.first()
                && !matches!(root.as_str(), "self" | "super" | "crate")
            {
                out.glob_crates.insert(root.clone());
            }
        }
        UseTree::Group(node) => {
            for item in &node.items {
                collect_tree(item, prefix, out);
            }
        }
    }
}

// needed helper: record one imported leaf name under its root crate
fn record(prefix: &[String], name: &str, out: &mut index::ExternalAliases) {
    let Some(root) = prefix.first() else {
        return;
    };
    if matches!(root.as_str(), "self" | "super" | "crate") {
        return;
    }
    out.local_to_crate.insert(name.to_string(), root.clone());
}

#[cfg(test)]
mod tests {
    use super::collect_external_aliases;

    #[test]
    fn test_usage() {
        let file: syn::File = syn::parse_str(
            "use quote::quote;\nuse syn::spanned::Spanned;\nuse syn::parse_str as parse;\nuse bevy::prelude::*;\nuse crate::index;\n",
        )
        .unwrap();
        let aliases = collect_external_aliases(&file);
        assert_eq!(
            aliases.local_to_crate.get("quote").map(String::as_str),
            Some("quote")
        );
        assert_eq!(
            aliases.local_to_crate.get("Spanned").map(String::as_str),
            Some("syn")
        );
        assert_eq!(
            aliases.local_to_crate.get("parse").map(String::as_str),
            Some("syn")
        );
        assert!(aliases.glob_crates.contains("bevy"));
        assert!(!aliases.local_to_crate.contains_key("index"));
    }
}
