use std::collections::BTreeMap;

use crate::index;
use crate::render;

pub fn render_dependency_tree_page(idx: &index::SymbolIndex, cfg: &render::LinkConfig) -> String {
    let graph = &idx.item_graph;
    let mut body = String::from("<h1>Dependencies</h1>");
    body.push_str(&format!(
        "<p class=\"muted\">{} code blocks &middot; {} links &middot; \
<span class=\"arrow\">&#9472;&#9472;is used by&#9472;&#9472;&#9654;</span></p>",
        graph.nodes.len(),
        graph.edges.len()
    ));
    body.push_str("<div class=\"cb-graph\"><svg class=\"cb-edges\" aria-hidden=\"true\"></svg>");

    let mut deps_of: Vec<Vec<usize>> = vec![Vec::new(); graph.nodes.len()];
    let mut users_of: Vec<Vec<usize>> = vec![Vec::new(); graph.nodes.len()];
    for edge in &graph.edges {
        if let Some(list) = deps_of.get_mut(edge.to) {
            list.push(edge.from);
        }
        if let Some(list) = users_of.get_mut(edge.from) {
            list.push(edge.to);
        }
    }

    let mut by_layer: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        by_layer.entry(node.layer).or_default().push(i);
    }
    let top = by_layer.keys().next_back().copied().unwrap_or(0);

    for (layer, nodes) in by_layer.iter().rev() {
        body.push_str(&format!(
            "<div class=\"layer-sep\"><span>{}</span></div>",
            layer_tag(*layer, top)
        ));
        body.push_str("<section class=\"layer\"><div class=\"layer-nodes\">");
        for &i in nodes {
            body.push_str(&pill_html(idx, i, &deps_of, &users_of));
        }
        body.push_str("</div></section>");
    }
    body.push_str("</div>");
    body.push_str(&edges_json(graph));

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

// needed helper: layer divider label, naming the base and the top layer
fn layer_tag(layer: usize, top: usize) -> String {
    if layer == 0 {
        "Layer 0 &middot; base".to_string()
    } else if layer == top {
        format!("Layer {layer} &middot; top")
    } else {
        format!("Layer {layer}")
    }
}

// needed helper: css class suffix for a code block kind
fn kind_class(kind: index::CodeBlock) -> &'static str {
    match kind {
        index::CodeBlock::Function => "kind-function",
        index::CodeBlock::Struct => "kind-struct",
        index::CodeBlock::Enum => "kind-enum",
        index::CodeBlock::Trait => "kind-trait",
    }
}

// needed helper: one pill node with magical externals and its is-used-by links
fn pill_html(
    idx: &index::SymbolIndex,
    i: usize,
    deps_of: &[Vec<usize>],
    users_of: &[Vec<usize>],
) -> String {
    let Some(node) = idx.item_graph.nodes.get(i) else {
        return String::new();
    };
    let empty: Vec<usize> = Vec::new();
    let deps = deps_of.get(i).unwrap_or(&empty);
    let users = users_of.get(i).unwrap_or(&empty);
    let has_ext = if node.external.is_empty() {
        ""
    } else {
        " has-ext"
    };
    let mut out = format!(
        "<div class=\"cb-wrap\" data-node=\"{id}\"><button class=\"cb-pill {kind}{has_ext}\" data-node=\"{id}\" aria-expanded=\"false\" title=\"{id}\">\
<span class=\"cb-dot\" aria-hidden=\"true\"></span><span class=\"cb-name\">{name}</span></button>",
        id = render::escape(&node.id),
        kind = kind_class(node.kind),
        has_ext = has_ext,
        name = render::escape(&node.name),
    );
    if !node.external.is_empty() {
        out.push_str("<div class=\"cb-ext\" aria-hidden=\"true\">");
        for ext in &node.external {
            out.push_str(&format!(
                "<span class=\"chip ext-chip\">{}</span>",
                render::escape(ext)
            ));
        }
        out.push_str("</div>");
    }
    out.push_str(&format!(
        "<details class=\"cb-detail\" id=\"cb-{key}\" data-key=\"cb-{key}\"><summary><span class=\"dep-count\">&#9472;&#9472;is used by&#9472;&#9472;&#9654;{uses}</span></summary><div class=\"dep-body\">",
        key = anchor(&node.id),
        uses = users.len(),
    ));
    push_links(idx, deps, "depends on", &mut out);
    push_links(idx, users, "is used by", &mut out);
    if deps.is_empty() && users.is_empty() && node.external.is_empty() {
        out.push_str("<p class=\"muted dep-row\">no dependencies</p>");
    }
    out.push_str("</div></details></div>");
    out
}

// needed helper: one labelled row of links to other code block pills
fn push_links(idx: &index::SymbolIndex, targets: &[usize], label: &str, out: &mut String) {
    if targets.is_empty() {
        return;
    }
    out.push_str(&format!(
        "<div class=\"dep-row\"><span class=\"dep-label\">{label}</span><div class=\"dep-uses\">"
    ));
    for &t in targets {
        if let Some(target) = idx.item_graph.nodes.get(t) {
            out.push_str(&format!(
                "<a class=\"dep-link\" href=\"#cb-{}\">{}</a>",
                anchor(&target.id),
                render::escape(&target.name)
            ));
        }
    }
    out.push_str("</div></div>");
}

// needed helper: edge list as json for the svg overlay
fn edges_json(graph: &index::ItemGraph) -> String {
    let mut out = String::from("<script type=\"application/json\" id=\"cb-edges\">[");
    let mut first = true;
    for edge in &graph.edges {
        let Some(from) = graph.nodes.get(edge.from) else {
            continue;
        };
        let Some(to) = graph.nodes.get(edge.to) else {
            continue;
        };
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str(&format!(
            "{{\"from\":\"{}\",\"to\":\"{}\"}}",
            render::escape(&from.id),
            render::escape(&to.id),
        ));
    }
    out.push_str("]</script>");
    out
}

// needed helper: html anchor id for a code block id
fn anchor(id: &str) -> String {
    if id.is_empty() {
        return "crate-root".to_string();
    }
    id.chars()
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
        idx.item_graph.nodes.push(index::ItemNode {
            id: "a::Cfg".to_string(),
            name: "Cfg".to_string(),
            kind: index::CodeBlock::Struct,
            external: vec!["std".to_string()],
            layer: 0,
        });
        idx.item_graph.nodes.push(index::ItemNode {
            id: "a::run".to_string(),
            name: "run".to_string(),
            kind: index::CodeBlock::Function,
            external: Vec::new(),
            layer: 1,
        });
        idx.item_graph
            .edges
            .push(index::ItemEdge { from: 0, to: 1 });
        idx.item_graph.externals.push("std".to_string());
        let cfg = render::LinkConfig {
            prefix: "/p/demo/".to_string(),
            assets: "/assets/".to_string(),
            html: false,
            nav: None,
        };
        let html = render_dependency_tree_page(&idx, &cfg);
        assert!(html.contains("Cfg"));
        assert!(html.contains("run"));
        assert!(html.contains("std"));
        assert!(html.contains("cb-pill kind-struct"));
        assert!(html.contains("cb-pill kind-function"));
        assert!(html.contains("Layer 0 &middot; base"));
        assert!(html.contains("is used by"));
        assert!(html.contains("id=\"cb-edges\""));
        let base = html.find("Layer 1 &middot; top").unwrap();
        let top = html.find("Layer 0 &middot; base").unwrap();
        assert!(base < top);
    }
}
