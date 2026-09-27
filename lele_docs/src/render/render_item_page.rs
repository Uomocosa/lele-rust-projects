use crate::index;
use crate::markdown;
use crate::render;

pub fn render_item_page(
    idx: &index::SymbolIndex,
    id: &str,
    cfg: &render::LinkConfig,
) -> Option<String> {
    let &i = idx.by_id.get(id)?;
    let item = idx.items.get(i)?;
    let mut body = String::new();
    body.push_str(&format!("<div class=\"kind\">{:?}</div>", item.kind));
    body.push_str(&format!("<h1>{}</h1>", render::escape(&item.name)));
    body.push_str(&format!(
        "<pre class=\"sig\">{}</pre>",
        render::escape(&item.signature)
    ));
    if let Some(doc) = &item.doc {
        body.push_str(&format!(
            "<div class=\"doc\">{}</div>",
            markdown::render_md(doc)
        ));
    }
    let rel = item.file.to_string_lossy();
    let url = render::href(cfg, render::LinkKind::File, &rel);
    body.push_str(&format!(
        "<p class=\"loc\"><a href=\"{url}#L{}-L{}\">{}</a>:{}-{}</p>",
        item.start_line,
        item.end_line,
        render::escape(&rel),
        item.start_line,
        item.end_line
    ));
    body.push_str(&refs(idx, cfg, "Callers", idx.callers.get(id)));
    body.push_str(&refs(idx, cfg, "Callees", idx.callees.get(id)));
    Some(render::page_shell(cfg, &item.name, &body))
}

// needed helper: render an id list as a titled list of item links
fn refs(
    idx: &index::SymbolIndex,
    cfg: &render::LinkConfig,
    title: &str,
    ids: Option<&Vec<String>>,
) -> String {
    let Some(ids) = ids else {
        return String::new();
    };
    if ids.is_empty() {
        return String::new();
    }
    let mut out = format!("<h2>{}</h2><ul class=\"list\">", render::escape(title));
    for rid in ids {
        let label = idx
            .by_id
            .get(rid)
            .and_then(|&j| idx.items.get(j))
            .map_or_else(|| rid.clone(), |item| item.id.clone());
        let url = render::href(cfg, render::LinkKind::Item, rid);
        out.push_str(&format!(
            "<li><a href=\"{url}\">{}</a></li>",
            render::escape(&label)
        ));
    }
    out.push_str("</ul>");
    out
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::render_item_page;
    use crate::index;
    use crate::render;

    #[test]
    fn test_usage() {
        let mut idx = index::SymbolIndex::default();
        idx.items.push(index::IndexItem {
            id: "a::f".to_string(),
            name: "f".to_string(),
            module: "a".to_string(),
            kind: index::ItemKind::Fn,
            file: PathBuf::from("src/a.rs"),
            start_line: 1,
            end_line: 1,
            name_line: 1,
            name_col_start: 0,
            name_col_end: 1,
            signature: "pub fn f".to_string(),
            doc: Some("does things".to_string()),
            delegates_to: None,
        });
        idx.by_id.insert("a::f".to_string(), 0);
        idx.callers
            .insert("a::f".to_string(), vec!["a::g".to_string()]);
        let cfg = render::LinkConfig {
            prefix: "/".to_string(),
            html: false,
        };
        let page = render_item_page(&idx, "a::f", &cfg).unwrap();
        assert!(page.contains("does things"));
        assert!(page.contains("Callers"));
    }
}
