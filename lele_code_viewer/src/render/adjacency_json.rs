use std::collections::BTreeMap;

use crate::index;
use crate::render;

pub fn adjacency_json(graph: &index::ItemGraph) -> String {
    let mut adj: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (n, edge) in graph.edges.iter().enumerate() {
        let Some(from) = graph.nodes.get(edge.from) else {
            continue;
        };
        let Some(to) = graph.nodes.get(edge.to) else {
            continue;
        };
        adj.entry(from.id.as_str()).or_default().push(n);
        if from.id != to.id {
            adj.entry(to.id.as_str()).or_default().push(n);
        }
    }
    let mut out = String::from("<script type=\"application/json\" id=\"cb-adj\">{");
    let mut first = true;
    for (id, edges) in &adj {
        if !first {
            out.push(',');
        }
        first = false;
        out.push('"');
        out.push_str(&render::escape(id));
        out.push_str("\":[");
        let mut edge_first = true;
        for e in edges {
            if !edge_first {
                out.push(',');
            }
            edge_first = false;
            out.push_str(&e.to_string());
        }
        out.push(']');
    }
    out.push_str("}</script>");
    out
}

#[cfg(test)]
mod tests {
    use super::adjacency_json;
    use crate::index;

    #[test]
    fn test_usage() {
        let graph = index::ItemGraph {
            nodes: vec![
                index::ItemNode {
                    id: "a::Cfg".to_string(),
                    name: "Cfg".to_string(),
                    kind: index::CodeBlock::Struct,
                    external: Vec::new(),
                    layer: 0,
                },
                index::ItemNode {
                    id: "a::run".to_string(),
                    name: "run".to_string(),
                    kind: index::CodeBlock::Function,
                    external: Vec::new(),
                    layer: 1,
                },
                index::ItemNode {
                    id: "a::solo".to_string(),
                    name: "solo".to_string(),
                    kind: index::CodeBlock::Function,
                    external: Vec::new(),
                    layer: 0,
                },
            ],
            edges: vec![
                index::ItemEdge { from: 0, to: 1 },
                index::ItemEdge { from: 9, to: 1 },
            ],
            externals: Vec::new(),
            groups: Vec::new(),
        };
        let html = adjacency_json(&graph);
        assert!(html.contains("id=\"cb-adj\""));
        assert!(html.contains("\"a::Cfg\":[0]"));
        assert!(html.contains("\"a::run\":[0]"));
        assert!(!html.contains("a::solo"));
    }
}
