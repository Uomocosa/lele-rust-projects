use crate::render;

pub fn page_shell(cfg: &render::LinkConfig, title: &str, body: &str) -> String {
    let css = render::href(cfg, render::LinkKind::Asset, "style.css");
    let js = render::href(cfg, render::LinkKind::Asset, "app.js");
    let home = render::href(cfg, render::LinkKind::Index, "");
    let search = render::href(cfg, render::LinkKind::Search, "");
    let (menu, label, tab_title) = match &cfg.nav {
        Some(nav) => (
            "<button class=\"btn menu\" id=\"menu-toggle\" aria-label=\"menu\" \
aria-controls=\"drawer\" aria-expanded=\"false\">&#9776;</button>",
            nav.name.as_str(),
            format!("{title} \u{b7} {}", nav.name),
        ),
        None => ("", "Lele Code Viewer", title.to_string()),
    };
    let drawer = drawer_html(cfg);
    let live = live_attrs(cfg);
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
<title>{tab_title}</title><link rel=\"stylesheet\" href=\"{css}\">\
<script defer src=\"{js}\"></script></head><body{live}>\
<header class=\"top\">{menu}\
<a class=\"home\" href=\"{home}\">{label}</a>\
<form class=\"search\" action=\"{search}\"><input name=\"q\" type=\"search\" placeholder=\"search\"></form>\
</header><div class=\"scrim\" id=\"scrim\"></div>{drawer}\
<main>{body}</main></body></html>",
        tab_title = render::escape(&tab_title),
        label = render::escape(label)
    )
}

// needed helper: slide-in navigation drawer (project switcher + the two tree icons)
fn drawer_html(cfg: &render::LinkConfig) -> String {
    let Some(nav) = &cfg.nav else {
        return String::new();
    };
    let mut out = String::from("<aside class=\"drawer\" id=\"drawer\">");
    out.push_str(
        "<div class=\"drawer-head\"><span>Lele Code Viewer</span>\
<a class=\"settings-link\" href=\"/settings\" aria-label=\"settings\" title=\"settings\">\
<svg viewBox=\"0 0 24 24\" width=\"18\" height=\"18\" aria-hidden=\"true\" fill=\"none\" \
stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\">\
<line x1=\"4\" y1=\"21\" x2=\"4\" y2=\"14\"/><line x1=\"4\" y1=\"10\" x2=\"4\" y2=\"3\"/>\
<line x1=\"12\" y1=\"21\" x2=\"12\" y2=\"12\"/><line x1=\"12\" y1=\"8\" x2=\"12\" y2=\"3\"/>\
<line x1=\"20\" y1=\"21\" x2=\"20\" y2=\"16\"/><line x1=\"20\" y1=\"12\" x2=\"20\" y2=\"3\"/>\
<line x1=\"1\" y1=\"14\" x2=\"7\" y2=\"14\"/><line x1=\"9\" y1=\"8\" x2=\"15\" y2=\"8\"/>\
<line x1=\"17\" y1=\"16\" x2=\"23\" y2=\"16\"/></svg></a></div>",
    );
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
            "<li><a class=\"{class}\" href=\"/p/{id}/\"><span class=\"proj-name\">{name}</span>\
<span class=\"proj-path\">{path}</span></a></li>",
            id = render::escape(&project.id),
            name = render::escape(&project.name),
            path = render::escape(&project.path)
        ));
    }
    out.push_str("</ul><nav class=\"views\">");
    out.push_str(&format!(
        "<a class=\"{}\" href=\"{}tree/file\"><span class=\"ico\">&#128450;</span>Files</a>",
        view_class(nav.view == render::ViewKind::Files),
        cfg.prefix
    ));
    out.push_str(&format!(
        "<a class=\"{}\" href=\"{}tree/deps\"><span class=\"ico\">&#128376;</span>Dependencies</a>",
        view_class(nav.view == render::ViewKind::Deps),
        cfg.prefix
    ));
    out.push_str(&format!(
        "</nav><div class=\"drawer-foot\"><div class=\"foot-name\">{}</div><div>{}</div></div></aside>",
        render::escape(&nav.name),
        render::escape(&nav.root)
    ));
    out
}

// needed helper: body data attributes that switch on live reload for this page
fn live_attrs(cfg: &render::LinkConfig) -> String {
    let Some(live) = cfg.nav.as_ref().and_then(|nav| nav.live.as_ref()) else {
        return String::new();
    };
    if live.watch.is_empty() {
        return String::new();
    }
    format!(
        " data-events=\"{}?since={}\" data-watch=\"{}\"",
        render::escape(&live.events),
        live.version,
        render::escape(&live.watch)
    )
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
        assert!(html.contains(">Lele Code Viewer</a>"));
        assert!(!html.contains("menu-toggle"));

        let with_nav = render::LinkConfig {
            prefix: "/p/demo/".to_string(),
            assets: "/assets/".to_string(),
            html: false,
            nav: Some(render::Nav {
                current: "demo".to_string(),
                name: "demo_crate".to_string(),
                root: "/home/u/demo".to_string(),
                projects: vec![render::NavProject {
                    id: "demo".to_string(),
                    name: "demo_crate".to_string(),
                    path: "~/code/demo".to_string(),
                }],
                view: render::ViewKind::Files,
                live: Some(render::Live {
                    events: "/p/demo/events".to_string(),
                    version: 7,
                    watch: "*".to_string(),
                    recent: std::collections::HashMap::new(),
                }),
            }),
        };
        let html = page_shell(&with_nav, "Files", "<p>x</p>");
        assert!(html.contains("menu-toggle"));
        assert!(html.contains("<a class=\"home\" href=\"/p/demo/\">demo_crate</a>"));
        assert!(html.contains("<title>Files \u{b7} demo_crate</title>"));
        assert!(html.contains("<a class=\"view active\" href=\"/p/demo/tree/file\">"));
        assert!(html.contains("Dependencies</a>"));
        assert!(html.contains("/home/u/demo"));
        assert!(html.contains("<span class=\"proj-path\">~/code/demo</span>"));
        assert!(html.contains("href=\"/settings\""));
        assert!(html.contains("<body data-events=\"/p/demo/events?since=7\" data-watch=\"*\">"));
        assert!(!page_shell(&cfg, "x", "").contains("data-events"));
    }
}
