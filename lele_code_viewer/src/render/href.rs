use crate::render;

pub fn href(cfg: &render::LinkConfig, kind: render::LinkKind, value: &str) -> String {
    let ext = if cfg.html { ".html" } else { "" };
    match kind {
        render::LinkKind::Index => {
            if cfg.html {
                format!("{}index.html", cfg.prefix)
            } else {
                cfg.prefix.clone()
            }
        }
        render::LinkKind::Search => format!("{}search{ext}", cfg.prefix),
        render::LinkKind::Asset => format!("{}assets/{value}", cfg.assets),
        render::LinkKind::File => format!("{}file/{value}{ext}", cfg.prefix),
        render::LinkKind::Md => format!("{}md/{value}{ext}", cfg.prefix),
        render::LinkKind::Item => format!("{}item/{}{ext}", cfg.prefix, value.replace("::", "/")),
    }
}

#[cfg(test)]
mod tests {
    use crate::render;

    use super::href;

    #[test]
    fn test_usage() {
        let server = render::LinkConfig {
            prefix: "/".to_string(),
            assets: "/".to_string(),
            html: false,
            nav: None,
        };
        assert_eq!(
            href(&server, render::LinkKind::Item, "clicker::Config"),
            "/item/clicker/Config"
        );
        let exported = render::LinkConfig {
            prefix: "../../".to_string(),
            assets: "../../".to_string(),
            html: true,
            nav: None,
        };
        assert_eq!(
            href(&exported, render::LinkKind::File, "src/lib.rs"),
            "../../file/src/lib.rs.html"
        );
        assert_eq!(
            href(&exported, render::LinkKind::Index, ""),
            "../../index.html"
        );
    }
}
