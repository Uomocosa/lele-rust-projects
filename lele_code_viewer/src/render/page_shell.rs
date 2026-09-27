use crate::render;

pub fn page_shell(cfg: &render::LinkConfig, title: &str, body: &str) -> String {
    let css = render::href(cfg, render::LinkKind::Asset, "style.css");
    let js = render::href(cfg, render::LinkKind::Asset, "app.js");
    let home = render::href(cfg, render::LinkKind::Index, "");
    let search = render::href(cfg, render::LinkKind::Search, "");
    let drawer = drawer_html(cfg);
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
<title>{title}</title><link rel=\"stylesheet\" href=\"{css}\">\
<script defer src=\"{js}\"></script></head><body>\
<header class=\"top\"><button class=\"btn menu\" id=\"menu-toggle\" aria-label=\"menu\">&#9776;</button>\
<a class=\"home\" href=\"{home}\">{title}</a>\
<form class=\"search\" action=\"{search}\"><input name=\"q\" type=\"search\" placeholder=\"search\"></form>\
</header><div class=\"scrim\" id=\"scrim\"></div>{drawer}\
<main>{body}</main></body></html>",
        title = render::escape(title)
    )
}

// needed helper: slide-in navigation drawer (project switcher + the two tree icons)
fn drawer_html(cfg: &render::LinkConfig) -> String {
    let Some(nav) = &cfg.nav else {
        return String::new();
    };
    let mut out = String::from("<aside class=\"drawer\" id=\"drawer\">");
    out.push_str("<div class=\"drawer-head\">Lele Code Viewer</div>");
    out.push_str(
        "<input class=\"proj-filter\" id=\"proj-filter\" type=\"search\" placeholder=\"filter projects\">",
    );
    out.push_str("<ul class=\"projects\" id=\"projects\">");
    for project in &nav.projects {
        let class = if project.id == nav.current {
            "active"
        } else {
            ""
        };
        out.push_str(&format!(
            "<li><a class=\"{class}\" href=\"/p/{id}/\">{name}</a></li>",
            id = render::escape(&project.id),
            name = render::escape(&project.name)
        ));
    }
    out.push_str("</ul><div class=\"views\">");
    out.push_str(&format!(
        "<a class=\"{}\" href=\"{}tree/file\"><span class=\"ico\">&#128193;</span>Files</a>",
        view_class(nav.view == render::ViewKind::Files),
        cfg.prefix
    ));
    out.push_str(&format!(
        "<a class=\"{}\" href=\"{}tree/deps\"><span class=\"ico\">&#128376;</span>Deps</a>",
        view_class(nav.view == render::ViewKind::Deps),
        cfg.prefix
    ));
    out.push_str(&format!(
        "</div><div class=\"drawer-foot\">{}</div></aside>",
        render::escape(&nav.current)
    ));
    out
}

// needed helper: css class string for an active/inactive view icon
fn view_class(active: bool) -> &'static str {
    if active { "view active" } else { "view" }
}

#[cfg(test)]
mod tests {
    use super::page_shell;
    use crate::render;

    #[test]
    fn test_usage() {
        let cfg = render::LinkConfig {
            prefix: "/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        let html = page_shell(&cfg, "crate", "<p>x</p>");
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("assets/style.css"));

        let with_nav = render::LinkConfig {
            prefix: "/p/demo/".to_string(),
            assets: "/assets/".to_string(),
            html: false,
            nav: Some(render::Nav {
                current: "demo".to_string(),
                projects: vec![render::NavProject {
                    id: "demo".to_string(),
                    name: "demo".to_string(),
                }],
                view: render::ViewKind::Files,
            }),
        };
        let html = page_shell(&with_nav, "crate", "<p>x</p>");
        assert!(html.contains("menu-toggle"));
        assert!(html.contains("href=\"/p/demo/tree/file\""));
    }
}
