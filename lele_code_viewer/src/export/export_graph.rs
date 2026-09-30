use std::path::Path;

use crate::Error;
use crate::index;

pub fn export_graph(idx: &index::SymbolIndex, out: &Path) -> Result<(), Error> {
    std::fs::create_dir_all(out)?;
    let graph = &idx.item_graph;
    std::fs::write(out.join("tree.json"), json_pretty(&tree_value(graph))?)?;
    std::fs::write(out.join("groups.json"), json_pretty(&groups_value(graph))?)?;
    Ok(())
}

// needed helper: nodes and edges (dependency -> user) keyed by item id
fn tree_value(graph: &index::ItemGraph) -> serde_json::Value {
    let nodes: Vec<serde_json::Value> = graph
        .nodes
        .iter()
        .map(|node| {
            serde_json::json!({
                "id": node.id,
                "name": node.name,
                "kind": format!("{:?}", node.kind).to_lowercase(),
                "layer": node.layer,
                "externals": node.external,
            })
        })
        .collect();
    let edges: Vec<serde_json::Value> = graph
        .edges
        .iter()
        .filter_map(|edge| {
            let from = graph.nodes.get(edge.from)?;
            let to = graph.nodes.get(edge.to)?;
            Some(serde_json::json!({ "dependency": from.id, "user": to.id }))
        })
        .collect();
    serde_json::json!({
        "format": "lele_code_viewer/tree@1",
        "nodes": nodes,
        "edges": edges,
        "externals": graph.externals,
    })
}

// needed helper: groups with member ids, layer span and internal/external edge counts
fn groups_value(graph: &index::ItemGraph) -> serde_json::Value {
    serde_json::json!({
        "format": "lele_code_viewer/groups@2",
        "algorithm": "ego-splitting + louvain (connected refinement), undirected, hub-damped, near-duplicates merged",
        "max_groups_per_node": 3,
        "min_group_size": 3,
        "groups": groups_list(graph, &graph.groups),
        "exclusive_groups": groups_list(graph, &graph.exclusive_groups),
    })
}

// needed helper: one json entry per group with ids, layer span and edge counts
fn groups_list(graph: &index::ItemGraph, list: &[index::ItemGroup]) -> Vec<serde_json::Value> {
    list.iter()
        .enumerate()
        .map(|(n, group)| {
            let ids: Vec<&str> = group
                .iter()
                .filter_map(|&i| graph.nodes.get(i).map(|node| node.id.as_str()))
                .collect();
            let layers = group
                .iter()
                .filter_map(|&i| graph.nodes.get(i).map(|node| node.layer));
            let low = layers.clone().min().unwrap_or(0);
            let high = layers.max().unwrap_or(0);
            let internal = graph
                .edges
                .iter()
                .filter(|e| group.contains(&e.from) && group.contains(&e.to))
                .count();
            let boundary = graph
                .edges
                .iter()
                .filter(|e| group.contains(&e.from) != group.contains(&e.to))
                .count();
            serde_json::json!({
                "id": n,
                "members": ids,
                "layer_span": [low, high],
                "internal_edges": internal,
                "boundary_edges": boundary,
            })
        })
        .collect()
}

// needed helper: pretty json with a trailing newline
fn json_pretty(value: &serde_json::Value) -> Result<String, Error> {
    let mut text = serde_json::to_string_pretty(value).map_err(|e| Error::Server(e.to_string()))?;
    text.push('\n');
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::export_graph;
    use crate::index;

    #[test]
    fn test_usage() {
        let mut idx = index::SymbolIndex::default();
        for (i, name) in ["a", "b", "c"].iter().enumerate() {
            idx.item_graph.nodes.push(index::ItemNode {
                id: format!("m::{name}"),
                name: (*name).to_string(),
                kind: index::CodeBlock::Function,
                external: Vec::new(),
                layer: i,
            });
        }
        idx.item_graph
            .edges
            .push(index::ItemEdge { from: 0, to: 1 });
        idx.item_graph.groups.push(index::ItemGroup(vec![0, 1, 2]));
        let dir = tempfile::tempdir().unwrap();
        export_graph(&idx, dir.path()).unwrap();
        let tree = std::fs::read_to_string(dir.path().join("tree.json")).unwrap();
        let groups = std::fs::read_to_string(dir.path().join("groups.json")).unwrap();
        assert!(tree.contains("\"dependency\": \"m::a\""));
        assert!(groups.contains("\"m::c\""));
        assert!(groups.contains("\"internal_edges\": 1"));
    }
}
