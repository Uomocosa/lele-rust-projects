use std::collections::BTreeSet;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::path::PathBuf;

use crate::index;

pub fn build_module_graph<S: BuildHasher>(
    idx: &mut index::SymbolIndex,
    file_deps: &HashMap<PathBuf, index::FileDeps, S>,
) {
    let known: BTreeSet<String> = idx
        .files
        .keys()
        .map(|path| index::module_of(path))
        .collect();
    let modules: Vec<String> = known.iter().cloned().collect();
    let node_of: HashMap<String, usize> = modules
        .iter()
        .enumerate()
        .map(|(i, module)| (module.clone(), i))
        .collect();

    let mut nodes: Vec<index::ModuleNode> = modules
        .iter()
        .map(|module| index::ModuleNode {
            module: module.clone(),
            external: Vec::new(),
            layer: 0,
        })
        .collect();

    let mut edge_set: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut externals: BTreeSet<String> = BTreeSet::new();
    for (path, deps) in file_deps {
        let from_module = index::module_of(path);
        let Some(&from) = node_of.get(&from_module) else {
            continue;
        };
        for ext in &deps.external {
            externals.insert(ext.clone());
            if let Some(node) = nodes.get_mut(from)
                && !node.external.contains(ext)
            {
                node.external.push(ext.clone());
            }
        }
        for candidate in &deps.internal {
            if let Some(target) = match_module(candidate, &known)
                && let Some(&to) = node_of.get(&target)
                && to != from
            {
                edge_set.insert((from, to));
            }
        }
    }

    let mut edges: Vec<index::ModuleEdge> = edge_set
        .iter()
        .map(|&(from, to)| index::ModuleEdge { from, to })
        .collect();

    let layers = compute_layers(nodes.len(), &edges);
    for (i, node) in nodes.iter_mut().enumerate() {
        node.layer = layers.get(i).copied().unwrap_or(0);
    }

    edges.sort_by_key(|edge| (edge.from, edge.to));
    idx.module_graph = index::ModuleGraph {
        nodes,
        edges,
        externals: externals.into_iter().collect(),
    };
}

// needed helper: map an import path to the nearest known module by longest prefix
fn match_module(candidate: &str, known: &BTreeSet<String>) -> Option<String> {
    if candidate.is_empty() {
        return None;
    }
    if known.contains(candidate) {
        return Some(candidate.to_string());
    }
    let segments: Vec<&str> = candidate.split("::").collect();
    for take in (1..segments.len()).rev() {
        let prefix = segments.get(..take)?.join("::");
        if known.contains(&prefix) {
            return Some(prefix);
        }
    }
    None
}

// needed helper: longest-path layering, base (no internal deps) at level 0
fn compute_layers(node_count: usize, edges: &[index::ModuleEdge]) -> Vec<usize> {
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    for edge in edges {
        if let Some(list) = adjacency.get_mut(edge.from) {
            list.push(edge.to);
        }
    }
    let mut levels = vec![0usize; node_count];
    let mut state = vec![0u8; node_count];
    for node in 0..node_count {
        visit(node, &adjacency, &mut levels, &mut state);
    }
    levels
}

// needed helper: depth-first longest path with cycle guard
fn visit(node: usize, adjacency: &[Vec<usize>], levels: &mut [usize], state: &mut [u8]) {
    if state.get(node).copied().unwrap_or(2) != 0 {
        return;
    }
    if let Some(slot) = state.get_mut(node) {
        *slot = 1;
    }
    let mut best = 0usize;
    if let Some(deps) = adjacency.get(node).cloned() {
        for dep in deps {
            if state.get(dep).copied().unwrap_or(2) == 1 {
                continue;
            }
            visit(dep, adjacency, levels, state);
            let candidate = levels.get(dep).copied().unwrap_or(0).saturating_add(1);
            best = best.max(candidate);
        }
    }
    if let Some(slot) = levels.get_mut(node) {
        *slot = best;
    }
    if let Some(slot) = state.get_mut(node) {
        *slot = 2;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::build_module_graph;
    use crate::index;

    #[test]
    fn test_usage() {
        let mut idx = index::SymbolIndex::default();
        idx.files.insert(
            PathBuf::from("src/lib.rs"),
            "pub mod a;\npub mod b;\n".to_string(),
        );
        idx.files
            .insert(PathBuf::from("src/a.rs"), "use crate::b;\n".to_string());
        idx.files
            .insert(PathBuf::from("src/b.rs"), "use std::fmt;\n".to_string());

        let mut deps = HashMap::new();
        deps.insert(
            PathBuf::from("src/a.rs"),
            index::FileDeps {
                internal: vec!["b".to_string()],
                external: Vec::new(),
            },
        );
        deps.insert(
            PathBuf::from("src/b.rs"),
            index::FileDeps {
                internal: Vec::new(),
                external: vec!["std".to_string()],
            },
        );
        build_module_graph(&mut idx, &deps);
        let a = idx
            .module_graph
            .nodes
            .iter()
            .find(|n| n.module == "a")
            .unwrap();
        let b = idx
            .module_graph
            .nodes
            .iter()
            .find(|n| n.module == "b")
            .unwrap();
        assert!(a.layer > b.layer);
        assert!(idx.module_graph.externals.contains(&"std".to_string()));
    }
}
