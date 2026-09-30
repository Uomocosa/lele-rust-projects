use std::collections::BTreeMap;

use crate::index;
use crate::render;

pub fn render_dependency_tree_page(idx: &index::SymbolIndex, cfg: &render::LinkConfig) -> String {
    let graph = &idx.item_graph;
    let mut body = String::from("<h1>Dependencies</h1>");
    body.push_str(&format!(
        "<p class=\"muted\">{} code blocks &middot; {} links</p>",
        graph.nodes.len(),
        graph.edges.len()
    ));
    body.push_str(
        "<div class=\"cb-controls\"><label class=\"cb-toggle\"><input type=\"checkbox\" id=\"cb-show-layers\" checked> show layers</label>\
<label class=\"cb-toggle\"><input type=\"checkbox\" id=\"cb-multi-groups\" checked> a code block can belong to multiple groups</label>\
<button type=\"button\" class=\"cb-fit\" id=\"cb-fit\">fit</button></div>",
    );
    body.push_str("<div class=\"cb-scroll\"><div class=\"cb-graph\"><svg class=\"cb-groups\" aria-hidden=\"true\"></svg><svg class=\"cb-edges\" aria-hidden=\"true\"></svg>");

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
            body.push_str(&pill_html(idx, i, cfg));
        }
        body.push_str("</div></section>");
    }
    body.push_str("</div></div>");
    body.push_str(&edges_json(graph));
    body.push_str(&groups_json(graph, &graph.groups, "cb-groups"));
    body.push_str(&groups_json(
        graph,
        &graph.exclusive_groups,
        "cb-groups-single",
    ));
    body.push_str(&ext_edges_json(graph));
    body.push_str(&render::adjacency_json(graph));

    if !graph.externals.is_empty() {
        body.push_str(
            "<section class=\"layer externals\"><div class=\"layer-tag\">external crates</div><div class=\"chips\">",
        );
        for ext in &graph.externals {
            body.push_str(&format!(
                "<span class=\"chip\" data-ext=\"{}\">{}</span>",
                render::escape(ext),
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

// needed helper: one pill node carrying its external crate names for hover lines
fn pill_html(idx: &index::SymbolIndex, i: usize, cfg: &render::LinkConfig) -> String {
    let Some(node) = idx.item_graph.nodes.get(i) else {
        return String::new();
    };
    let has_ext = if node.external.is_empty() {
        ""
    } else {
        " has-ext"
    };
    let mut ext_attr = String::new();
    for ext in &node.external {
        if !ext_attr.is_empty() {
            ext_attr.push(',');
        }
        ext_attr.push_str(&render::escape(ext));
    }
    let href_attr = pill_href(idx, cfg, &node.id)
        .map_or_else(String::new, |href| format!(" data-href=\"{href}\""));
    format!(
        "<div class=\"cb-wrap\" data-node=\"{id}\" data-layer=\"{layer}\" data-exts=\"{exts}\"{href}><button class=\"cb-pill {kind}{has_ext}\" data-node=\"{id}\" title=\"{id}\">\
<span class=\"cb-dot\" aria-hidden=\"true\"></span><span class=\"cb-name\">{name}</span></button></div>",
        id = render::escape(&node.id),
        layer = node.layer,
        exts = ext_attr,
        href = href_attr,
        kind = kind_class(node.kind),
        has_ext = has_ext,
        name = render::escape(&node.name),
    )
}

// needed helper: file anchor for a dependency pill, pointing at its definition block
fn pill_href(idx: &index::SymbolIndex, cfg: &render::LinkConfig, id: &str) -> Option<String> {
    let &i = idx.by_id.get(id)?;
    let item = idx.items.get(i)?;
    let rel = item.file.to_string_lossy();
    let base = render::href(cfg, render::LinkKind::File, &rel);
    Some(format!("{base}#L{}-L{}", item.start_line, item.end_line))
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

// needed helper: detected domain groups as json arrays of node ids
fn groups_json(graph: &index::ItemGraph, groups: &[index::ItemGroup], id: &str) -> String {
    let mut out = format!("<script type=\"application/json\" id=\"{id}\">[");
    for (n, group) in groups.iter().enumerate() {
        if n > 0 {
            out.push(',');
        }
        out.push('[');
        let mut first = true;
        for &i in group.iter() {
            let Some(node) = graph.nodes.get(i) else {
                continue;
            };
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str(&format!("\"{}\"", render::escape(&node.id)));
        }
        out.push(']');
    }
    out.push_str("]</script>");
    out
}

// needed helper: external edge list as json for hover-only svg lines
fn ext_edges_json(graph: &index::ItemGraph) -> String {
    let mut out = String::from("<script type=\"application/json\" id=\"cb-ext-edges\">[");
    let mut first = true;
    for node in &graph.nodes {
        for ext in &node.external {
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str(&format!(
                "{{\"from\":\"{}\",\"ext\":\"{}\"}}",
                render::escape(&node.id),
                render::escape(ext),
            ));
        }
    }
    out.push_str("]</script>");
    out
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::render_dependency_tree_page;
    use crate::index;
    use crate::render;

    #[test]
    fn test_usage() {
        let mut idx = index::SymbolIndex::default();
        for (id, name, kind, start, end) in [
            ("a::Cfg", "Cfg", index::ItemKind::Struct, 3, 7),
            ("a::run", "run", index::ItemKind::Fn, 10, 14),
        ] {
            idx.items.push(index::IndexItem {
                id: id.to_string(),
                name: name.to_string(),
                module: "a".to_string(),
                kind,
                file: PathBuf::from("src/a.rs"),
                start_line: start,
                end_line: end,
                name_line: start,
                name_col_start: 0,
                name_col_end: 1,
                signature: name.to_string(),
                doc: None,
                delegates_to: None,
                is_test: false,
                external: Vec::new(),
            });
        }
        idx.by_id.insert("a::Cfg".to_string(), 0);
        idx.by_id.insert("a::run".to_string(), 1);
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
        assert!(!html.contains("is used by"));
        assert!(html.contains("id=\"cb-edges\""));
        assert!(html.contains("id=\"cb-ext-edges\""));
        assert!(html.contains("id=\"cb-adj\""));
        assert!(html.contains("id=\"cb-groups\">[]"));
        assert!(html.contains("id=\"cb-groups-single\">[]"));
        assert!(html.contains("id=\"cb-multi-groups\""));
        assert!(html.contains("data-layer=\"1\""));
        assert!(html.contains("id=\"cb-show-layers\""));
        assert!(html.contains("data-ext=\"std\""));
        assert!(html.contains("data-href=\"/p/demo/file/src/a.rs#L3-L7\""));
        assert!(html.contains("data-href=\"/p/demo/file/src/a.rs#L10-L14\""));
        let base = html.find("Layer 1 &middot; top").unwrap();
        let top = html.find("Layer 0 &middot; base").unwrap();
        assert!(base < top);
    }
}
