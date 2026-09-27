use std::collections::BTreeMap;

use crate::index;
use crate::render;

pub fn render_dependency_tree_page(idx: &index::SymbolIndex, cfg: &render::LinkConfig) -> String {
    let graph = &idx.module_graph;
    let mut body = String::from("<h1>Dependencies</h1>");
    body.push_str(&format!(
        "<p class=\"muted\">{} modules &middot; {} links &middot; \
<span class=\"arrow\">&rarr;</span> uses &middot; <span class=\"arrow\">&larr;</span> used by</p>",
        graph.nodes.len(),
        graph.edges.len()
    ));

    let mut uses: Vec<Vec<usize>> = vec![Vec::new(); graph.nodes.len()];
    let mut used_by: Vec<Vec<usize>> = vec![Vec::new(); graph.nodes.len()];
    for edge in &graph.edges {
        if let Some(list) = uses.get_mut(edge.from) {
            list.push(edge.to);
        }
        if let Some(list) = used_by.get_mut(edge.to) {
            list.push(edge.from);
        }
    }

    let mut by_layer: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        by_layer.entry(node.layer).or_default().push(i);
    }
    let top = by_layer.keys().next_back().copied().unwrap_or(0);

    for (layer, nodes) in by_layer.iter().rev() {
        body.push_str(&format!(
            "<section class=\"layer\"><div class=\"layer-tag\">{}</div><div class=\"layer-nodes\">",
            layer_tag(*layer, top)
        ));
        for &i in nodes {
            body.push_str(&node_html(idx, i, &uses, &used_by));
        }
        body.push_str("</div></section>");
    }

    if !graph.externals.is_empty() {
        body.push_str(
            "<section class=\"layer externals\"><div class=\"layer-tag\">external crates</div><div class=\"chips\">",
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

// needed helper: layer heading, naming the base and the top layer
fn layer_tag(layer: usize, top: usize) -> String {
    if layer == 0 {
        "L0 &middot; base".to_string()
    } else if layer == top {
        format!("L{layer} &middot; top")
    } else {
        format!("L{layer}")
    }
}

// needed helper: one collapsible module node with its dependency links both ways
fn node_html(
    idx: &index::SymbolIndex,
    i: usize,
    uses: &[Vec<usize>],
    used_by: &[Vec<usize>],
) -> String {
    let Some(node) = idx.module_graph.nodes.get(i) else {
        return String::new();
    };
    let empty: Vec<usize> = Vec::new();
    let out_links = uses.get(i).unwrap_or(&empty);
    let in_links = used_by.get(i).unwrap_or(&empty);
    let mut out = format!(
        "<details class=\"dep-node\" id=\"m-{}\"><summary><span class=\"dep-name\">{}</span>\
<span class=\"dep-count\">&rarr;{} &larr;{}</span></summary><div class=\"dep-body\">",
        anchor(&node.module),
        render::escape(&module_label(&node.module)),
        out_links.len(),
        in_links.len()
    );
    push_links(idx, out_links, "&rarr;", "uses", &mut out);
    push_links(idx, in_links, "&larr;", "used by", &mut out);
    if !node.external.is_empty() {
        out.push_str(
            "<div class=\"dep-row\"><span class=\"dep-label\">crates</span><div class=\"chips\">",
        );
        for ext in &node.external {
            out.push_str(&format!(
                "<span class=\"chip\">{}</span>",
                render::escape(ext)
            ));
        }
        out.push_str("</div></div>");
    }
    if out_links.is_empty() && in_links.is_empty() && node.external.is_empty() {
        out.push_str("<p class=\"muted dep-row\">no dependencies</p>");
    }
    out.push_str("</div></details>");
    out
}

// needed helper: one labelled row of links to other module nodes
fn push_links(
    idx: &index::SymbolIndex,
    targets: &[usize],
    arrow: &str,
    label: &str,
    out: &mut String,
) {
    if targets.is_empty() {
        return;
    }
    out.push_str(&format!(
        "<div class=\"dep-row\"><span class=\"dep-label\">{label}</span><div class=\"dep-uses\">"
    ));
    for &t in targets {
        if let Some(target) = idx.module_graph.nodes.get(t) {
            out.push_str(&format!(
                "<a class=\"dep-link\" href=\"#m-{}\">{arrow} {}</a>",
                anchor(&target.module),
                render::escape(&module_label(&target.module))
            ));
        }
    }
    out.push_str("</div></div>");
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
        assert!(html.contains("<details class=\"dep-node\" id=\"m-app\">"));
        assert!(html.contains("href=\"#m-base\">&rarr; base</a>"));
        assert!(html.contains("href=\"#m-app\">&larr; app</a>"));
        assert!(html.contains("L0 &middot; base"));
        let top = html.find("L1 &middot; top").unwrap();
        let base = html.find("L0 &middot; base").unwrap();
        let crates = html.find("external crates").unwrap();
        assert!(top < base && base < crates);
    }
}
