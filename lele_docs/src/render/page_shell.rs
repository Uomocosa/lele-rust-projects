use crate::render;

pub fn page_shell(cfg: &render::LinkConfig, title: &str, body: &str) -> String {
    let css = render::href(cfg, render::LinkKind::Asset, "style.css");
    let js = render::href(cfg, render::LinkKind::Asset, "app.js");
    let home = render::href(cfg, render::LinkKind::Index, "");
    let search = render::href(cfg, render::LinkKind::Search, "");
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
<title>{title}</title><link rel=\"stylesheet\" href=\"{css}\">\
<script defer src=\"{js}\"></script></head><body>\
<header class=\"top\"><a class=\"btn\" href=\"javascript:history.back()\">&#8592;</a>\
<a class=\"home\" href=\"{home}\">{title}</a>\
<form class=\"search\" action=\"{search}\"><input name=\"q\" type=\"search\" placeholder=\"search\"></form>\
</header><main>{body}</main></body></html>",
        title = render::escape(title)
    )
}

#[cfg(test)]
mod tests {
    use super::page_shell;
    use crate::render;

    #[test]
    fn test_usage() {
        let cfg = render::LinkConfig {
            prefix: "/".to_string(),
            html: false,
        };
        let html = page_shell(&cfg, "crate", "<p>x</p>");
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("assets/style.css"));
    }
}
