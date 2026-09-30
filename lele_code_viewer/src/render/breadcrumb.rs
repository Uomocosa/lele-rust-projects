use crate::render;

pub fn breadcrumb(rel: &str, cfg: &render::LinkConfig) -> String {
    let mut out = String::from("<h1 class=\"path\">");
    let parts: Vec<&str> = rel.split('/').collect();
    if parts.len() < 2 {
        out.push_str(&render::escape(rel));
        out.push_str("</h1>");
        return out;
    }
    let last = parts.len().saturating_sub(1);
    for (i, part) in parts.iter().enumerate() {
        if i == last {
            out.push_str(&render::escape(part));
        } else {
            let prefix = parts
                .iter()
                .take(i.saturating_add(1))
                .copied()
                .collect::<Vec<_>>()
                .join("/");
            let url = format!("{}tree/file#{}", cfg.prefix, prefix);
            out.push_str(&format!(
                "<a href=\"{}\">{}</a>/",
                render::escape(&url),
                render::escape(part)
            ));
        }
    }
    out.push_str("</h1>");
    out
}

#[cfg(test)]
mod tests {
    use super::breadcrumb;
    use crate::render;

    #[test]
    fn test_usage() {
        let cfg = render::LinkConfig {
            prefix: "/p/demo/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        let html = breadcrumb("src/cdp/call.rs", &cfg);
        assert!(html.contains("<a href=\"/p/demo/tree/file#src\">src</a>/"));
        assert!(html.contains("<a href=\"/p/demo/tree/file#src/cdp\">cdp</a>/"));
        assert!(html.contains("call.rs</h1>"));
        assert!(!html.contains("<a href=\"/p/demo/tree/file#src/cdp/call.rs\""));
        let single = breadcrumb("Cargo.toml", &cfg);
        assert!(!single.contains("<a "));
        assert!(single.contains("Cargo.toml"));
    }
}
