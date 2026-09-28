use crate::render;

pub fn render_settings_page(cfg: &render::LinkConfig, view: &render::SettingsView) -> String {
    let mut body = String::from("<div class=\"settings\"><h1>Settings</h1>");
    if let Some(notice) = &view.notice {
        body.push_str(&format!(
            "<p class=\"notice\">{}</p>",
            render::escape(notice)
        ));
    }
    if let Some(error) = &view.error {
        body.push_str(&format!(
            "<p class=\"notice error\">{}</p>",
            render::escape(error)
        ));
    }
    let status = if view.scanning {
        format!(
            "{} projects listed &middot; rescanning&hellip;",
            view.project_count
        )
    } else {
        format!("{} projects listed", view.project_count)
    };
    body.push_str(&format!(
        "<h2>Ignored folders</h2>\
<p class=\"muted\">One regular expression per line, matched against each folder's full path. \
Matching folders (and everything inside them) are skipped when looking for crates.</p>\
<form class=\"panel\" method=\"post\" action=\"/settings\">\
<textarea name=\"ignore\" rows=\"8\" spellcheck=\"false\" autocapitalize=\"off\" \
aria-label=\"ignore rules\">{rules}</textarea>\
<div class=\"actions\"><button class=\"primary\" type=\"submit\" name=\"action\" value=\"save\">\
Save &amp; rescan</button><button type=\"submit\" name=\"action\" value=\"defaults\">\
Reset to defaults</button></div></form>\
<p class=\"muted\">{status} &middot; saved in <code>{path}</code></p>",
        rules = render::escape(&view.rules),
        path = render::escape(&view.settings_path),
    ));
    body.push_str(
        "<h2>Maintenance</h2><div class=\"panel actions\">\
<form method=\"post\" action=\"/settings/rescan\"><button type=\"submit\">Refresh project list</button></form>",
    );
    if view.self_update {
        body.push_str(
            "<form method=\"post\" action=\"/settings/update\"><button type=\"submit\">\
Update &amp; restart viewer</button></form></div>\
<p class=\"muted\">Update rebuilds the viewer from source (<code>lele:service:update</code>) \
and restarts it; the page is unavailable for about a minute.</p>",
        );
    } else {
        body.push_str("</div>");
    }
    body.push_str("</div>");
    render::page_shell(cfg, "Settings", &body)
}

#[cfg(test)]
mod tests {
    use super::render_settings_page;
    use crate::render;

    #[test]
    fn test_usage() {
        let cfg = render::LinkConfig {
            prefix: "/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        let view = render::SettingsView {
            rules: "(^|/)target(/|$)\n<x>".to_string(),
            error: Some("rule 2 bad".to_string()),
            notice: None,
            project_count: 12,
            scanning: true,
            self_update: false,
            settings_path: "~/.config/lele-code-viewer/settings.toml".to_string(),
        };
        let html = render_settings_page(&cfg, &view);
        assert!(html.contains("(^|/)target(/|$)\n&lt;x&gt;</textarea>"));
        assert!(html.contains("rule 2 bad"));
        assert!(html.contains("12 projects listed &middot; rescanning"));
        assert!(html.contains("action=\"/settings/rescan\""));
        assert!(!html.contains("/settings/update"));
    }
}
