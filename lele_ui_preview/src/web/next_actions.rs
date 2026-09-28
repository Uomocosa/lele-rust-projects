use std::collections::BTreeSet;

use crate::web;

pub fn next_actions(probe: &web::Probe, target: &web::Target) -> Vec<web::Action> {
    let current = format!("{}{}", probe.url, probe.hash);
    let mut groups = BTreeSet::new();
    let mut actions = Vec::new();
    for element in &probe.elements {
        if target
            .skip_actions
            .iter()
            .any(|skip| element.label.contains(skip.as_str()))
        {
            continue;
        }
        if !groups.insert(web::element_group(element, &target.patterns)) {
            continue;
        }
        if let Some(action) = element_action(probe, target, element, &current) {
            actions.push(action);
        }
    }
    for key in &target.keys {
        actions.push(web::Action::Key { key: key.clone() });
    }
    if probe.scrollable && !probe.scrolled {
        actions.push(web::Action::ScrollBottom);
    }
    actions
}

// needed helper: the one representative action for an interactive element
fn element_action(
    probe: &web::Probe,
    target: &web::Target,
    element: &web::ProbeElement,
    current: &str,
) -> Option<web::Action> {
    let label = element.label.clone();
    if let Some(href) = &element.href {
        if href.starts_with('#') {
            return Some(web::Action::Click {
                index: element.index,
                label,
            });
        }
        let location = element.absolute.as_deref()?.strip_prefix(&probe.origin)?;
        let path = location.split(['?', '#']).next().unwrap_or(location);
        if target
            .skip_paths
            .iter()
            .any(|skip| path.starts_with(skip.as_str()))
        {
            return None;
        }
        if !target.patterns.is_empty() && web::match_route(&target.patterns, path).is_none() {
            return None;
        }
        if location == current {
            return None;
        }
        return Some(web::Action::Navigate {
            location: location.to_string(),
            label,
        });
    }
    match (element.tag.as_str(), element.input_type.as_deref()) {
        (_, Some("text" | "search" | "email" | "url")) if !probe.typed => Some(web::Action::Fill {
            index: element.index,
            text: target.fill_text.clone(),
            label,
        }),
        ("select", _) | (_, Some(_)) => None,
        _ => Some(web::Action::Click {
            index: element.index,
            label,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::next_actions;
    use crate::web;

    fn element(index: usize, group: &str, tag: &str, href: Option<&str>) -> web::ProbeElement {
        web::ProbeElement {
            index,
            group: group.to_string(),
            tag: tag.to_string(),
            href: href.map(str::to_string),
            absolute: href.map(|h| format!("http://h{h}")),
            input_type: if tag == "input" {
                Some("search".to_string())
            } else {
                None
            },
            label: format!("el{index}"),
        }
    }

    #[test]
    fn test_usage() {
        let probe = web::Probe {
            url: "/".to_string(),
            hash: String::new(),
            origin: "http://h".to_string(),
            body_class: String::new(),
            scrolled: false,
            scrollable: true,
            typed: false,
            elements: vec![
                element(0, "button#menu-toggle", "button", None),
                element(1, "ul>li>a", "a", Some("/p/a")),
                element(2, "ul>li>a", "a", Some("/p/b")),
                element(3, "input#proj-filter", "input", None),
                element(4, "link", "a", Some("/assets/app.js")),
                element(5, "form>button", "button", None),
            ],
        };
        let target = web::Target {
            base_url: "http://h".to_string(),
            patterns: vec!["/".to_string(), "/p/{id}".to_string()],
            keys: vec!["Escape".to_string()],
            skip_paths: Vec::new(),
            skip_actions: vec!["el5".to_string()],
            fill_text: "src".to_string(),
            settle_ms: 0,
            max_states: 10,
            max_depth: 2,
        };
        let actions = next_actions(&probe, &target);
        assert_eq!(actions.len(), 5);
        assert!(matches!(
            actions.first(),
            Some(web::Action::Click { index: 0, .. })
        ));
        assert!(actions.contains(&web::Action::Navigate {
            location: "/p/a".to_string(),
            label: "el1".to_string()
        }));
        assert!(actions.contains(&web::Action::ScrollBottom));
    }
}
