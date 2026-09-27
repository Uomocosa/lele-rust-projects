use std::collections::BTreeMap;

use crate::index;
use crate::render;

pub fn render_dependency_tree_page(idx: &index::SymbolIndex, cfg: &render::LinkConfig) -> String {
    let graph = &idx.module_graph;
    let mut body = String::from("<h1>Dependencies</h1>");
    body.push_str(&format!(
        "<p class=\"muted\">{} modules &middot; {} links</p>",
        graph.nodes.len(),
        graph.edges.len()
    ));

    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); graph.nodes.len()];
    for edge in &graph.edges {
        if let Some(list) = adjacency.get_mut(edge.from) {
            list.push(edge.to);
        }
    }

    let mut by_layer: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        by_layer.entry(node.layer).or_default().push(i);
    }

    for (layer, nodes) in by_layer.iter().rev() {
        body.push_str(&format!(
            "<section class=\"layer\"><div class=\"layer-tag\">L{layer}</div><div class=\"layer-nodes\">"
        ));
        for &i in nodes {
            body.push_str(&node_html(idx, i, &adjacency));
        }
        body.push_str("</div></section>");
    }

    if !graph.externals.is_empty() {
        body.push_str(
            "<section class=\"layer externals\"><div class=\"layer-tag\">crates</div><div class=\"chips\">",
        );
        for ext in &graph.externals {
            body.push_str(&format!(
                "<span class=\"chip\">{}</span>",
                render::escape(ext)
            ));
        }
        body.push_str("</div></section>");
    }

    render::page_shell(cfg, "Dependencies", &body)
}

// needed helper: one module node with its outgoing dependency links
fn node_html(idx: &index::SymbolIndex, i: usize, adjacency: &[Vec<usize>]) -> String {
    let Some(node) = idx.module_graph.nodes.get(i) else {
        return String::new();
    };
    let label = module_label(&node.module);
    let mut out = format!(
        "<div class=\"dep-node\" id=\"m-{}\"><div class=\"dep-name\">{}</div>",
        anchor(&node.module),
        render::escape(&label)
    );
    let empty: Vec<usize> = Vec::new();
    let deps = adjacency.get(i).unwrap_or(&empty);
    if !deps.is_empty() {
        out.push_str("<div class=\"dep-uses\">");
        for &d in deps {
            if let Some(target) = idx.module_graph.nodes.get(d) {
                out.push_str(&format!(
                    "<a class=\"dep-link\" href=\"#m-{}\">{}</a>",
                    anchor(&target.module),
                    render::escape(&module_label(&target.module))
                ));
            }
        }
        out.push_str("</div>");
    }
    if !node.external.is_empty() {
        out.push_str("<div class=\"chips\">");
        for ext in &node.external {
            out.push_str(&format!(
                "<span class=\"chip\">{}</span>",
                render::escape(ext)
            ));
        }
        out.push_str("</div>");
    }
    out.push_str("</div>");
    out
}

// needed helper: human label for a module path
fn module_label(module: &str) -> String {
    if module.is_empty() {
        "(crate root)".to_string()
    } else {
        module.to_string()
    }
}

// needed helper: html anchor id for a module path
fn anchor(module: &str) -> String {
    if module.is_empty() {
        return "crate-root".to_string();
    }
    module
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::render_dependency_tree_page;
    use crate::index;
    use crate::render;

    #[test]
    fn test_usage() {
        let mut idx = index::SymbolIndex::default();
        idx.module_graph.nodes.push(index::ModuleNode {
            module: "base".to_string(),
            external: vec!["std".to_string()],
            layer: 0,
        });
        idx.module_graph.nodes.push(index::ModuleNode {
            module: "app".to_string(),
            external: Vec::new(),
            layer: 1,
        });
        idx.module_graph
            .edges
            .push(index::ModuleEdge { from: 1, to: 0 });
        idx.module_graph.externals.push("std".to_string());
        let cfg = render::LinkConfig {
            prefix: "/p/demo/".to_string(),
            assets: "/assets/".to_string(),
            html: false,
            nav: None,
        };
        let html = render_dependency_tree_page(&idx, &cfg);
        assert!(html.contains("base"));
        assert!(html.contains("app"));
        assert!(html.contains("std"));
    }
}
