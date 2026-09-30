use std::collections::BTreeSet;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::path::PathBuf;

use crate::index;

pub fn build_item_graph<S: BuildHasher>(
    idx: &mut index::SymbolIndex,
    file_deps: &HashMap<PathBuf, index::FileDeps, S>,
) {
    let mut node_of: HashMap<String, usize> = HashMap::new();
    let mut nodes: Vec<index::ItemNode> = Vec::new();
    for item in &idx.items {
        let Some(kind) = kind_of(item.kind) else {
            continue;
        };
        if node_of.contains_key(&item.id) {
            continue;
        }
        node_of.insert(item.id.clone(), nodes.len());
        nodes.push(index::ItemNode {
            id: item.id.clone(),
            name: item.name.clone(),
            kind,
            external: Vec::new(),
            layer: 0,
        });
    }

    let mut edge_set: BTreeSet<(usize, usize)> = BTreeSet::new();
    for (caller, callees) in &idx.callees {
        let Some(&user) = node_of.get(caller) else {
            continue;
        };
        for callee in callees {
            let Some(&dep) = node_of.get(callee) else {
                continue;
            };
            if dep != user {
                edge_set.insert((dep, user));
            }
        }
    }
    for (id, &user) in &node_of {
        let Some(parent) = parent_of(id) else {
            continue;
        };
        if let Some(&dep) = node_of.get(&parent)
            && dep != user
        {
            edge_set.insert((dep, user));
        }
    }

    let mut externals: BTreeSet<String> = BTreeSet::new();
    for (path, deps) in file_deps {
        let Some(members) = idx.items_by_file.get(path) else {
            continue;
        };
        for &item_idx in members {
            let Some(item) = idx.items.get(item_idx) else {
                continue;
            };
            let Some(&node) = node_of.get(&item.id) else {
                continue;
            };
            let Some(slot) = nodes.get_mut(node) else {
                continue;
            };
            for ext in &deps.external {
                externals.insert(ext.clone());
                if !slot.external.contains(ext) {
                    slot.external.push(ext.clone());
                }
            }
        }
    }

    let pairs: Vec<(usize, usize)> = edge_set.iter().map(|&(dep, user)| (user, dep)).collect();
    let layers = index::compute_layers(nodes.len(), &pairs);
    for (i, node) in nodes.iter_mut().enumerate() {
        node.layer = layers.get(i).copied().unwrap_or(0);
        node.external.sort();
    }

    let mut edges: Vec<index::ItemEdge> = edge_set
        .iter()
        .map(|&(from, to)| index::ItemEdge { from, to })
        .collect();
    edges.sort_by_key(|edge| (edge.from, edge.to));
    idx.item_graph = index::ItemGraph {
        nodes,
        edges,
        externals: externals.into_iter().collect(),
    };
}

// needed helper: keep only brace-defined items, drop const/static/alias/mod
fn kind_of(kind: index::ItemKind) -> Option<index::CodeBlock> {
    match kind {
        index::ItemKind::Fn => Some(index::CodeBlock::Function),
        index::ItemKind::Struct => Some(index::CodeBlock::Struct),
        index::ItemKind::Enum => Some(index::CodeBlock::Enum),
        index::ItemKind::Trait => Some(index::CodeBlock::Trait),
        index::ItemKind::Const
        | index::ItemKind::Static
        | index::ItemKind::TypeAlias
        | index::ItemKind::Mod => None,
    }
}

// needed helper: enclosing scope of an item id (method -> parent type)
fn parent_of(id: &str) -> Option<String> {
    let (parent, _leaf) = id.rsplit_once("::")?;
    if parent.is_empty() {
        return None;
    }
    Some(parent.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::build_item_graph;
    use crate::index;

    #[test]
    fn test_usage() {
        let mut idx = index::SymbolIndex::default();
        let file = PathBuf::from("src/a.rs");
        for (id, name, kind) in [
            ("a::Cfg", "Cfg", index::ItemKind::Struct),
            ("a::Cfg::new", "new", index::ItemKind::Fn),
            ("a::run", "run", index::ItemKind::Fn),
            ("a::LIMIT", "LIMIT", index::ItemKind::Const),
            ("a::Alias", "Alias", index::ItemKind::TypeAlias),
        ] {
            idx.items.push(index::IndexItem {
                id: id.to_string(),
                name: name.to_string(),
                module: "a".to_string(),
                kind,
                file: file.clone(),
                start_line: 1,
                end_line: 3,
                name_line: 1,
                name_col_start: 0,
                name_col_end: 1,
                signature: name.to_string(),
                doc: None,
                delegates_to: None,
            });
        }
        idx.items_by_file.insert(file.clone(), vec![0, 1, 2, 3, 4]);
        idx.callees
            .insert("a::run".to_string(), vec!["a::Cfg".to_string()]);
        let mut deps = HashMap::new();
        deps.insert(
            file,
            index::FileDeps {
                internal: Vec::new(),
                external: vec!["serde".to_string()],
            },
        );
        build_item_graph(&mut idx, &deps);
        let graph = &idx.item_graph;
        assert_eq!(graph.nodes.len(), 3);
        assert!(
            graph
                .nodes
                .iter()
                .all(|n| n.external == vec!["serde".to_string()])
        );
        let cfg = graph.nodes.iter().find(|n| n.id == "a::Cfg").unwrap();
        let new = graph.nodes.iter().find(|n| n.id == "a::Cfg::new").unwrap();
        let run = graph.nodes.iter().find(|n| n.id == "a::run").unwrap();
        assert!(new.layer > cfg.layer);
        assert!(run.layer > cfg.layer);
        assert!(graph.edges.iter().any(|e| {
            let dep = graph.nodes.get(e.from).map(|n| n.id.as_str()).unwrap_or("");
            let user = graph.nodes.get(e.to).map(|n| n.id.as_str()).unwrap_or("");
            dep == "a::Cfg" && user == "a::run"
        }));
        assert!(graph.externals.contains(&"serde".to_string()));
    }
}
