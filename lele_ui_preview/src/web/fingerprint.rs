use std::collections::BTreeSet;

use crate::web;

pub fn fingerprint(screen: &str, probe: &web::Probe, patterns: &[String]) -> String {
    let mut classes: Vec<&str> = probe.body_class.split_whitespace().collect();
    classes.sort_unstable();
    let groups: BTreeSet<String> = probe
        .elements
        .iter()
        .map(|el| web::element_group(el, patterns))
        .collect();
    let digest = blake3::hash(groups.into_iter().collect::<Vec<_>>().join("\n").as_bytes());
    let short: String = digest.to_hex().chars().take(12).collect();
    format!(
        "{screen}|hash={}|body={}|scrolled={}|typed={}|ui={short}",
        !probe.hash.is_empty(),
        classes.join("."),
        probe.scrolled,
        probe.typed
    )
}

#[cfg(test)]
mod tests {
    use super::fingerprint;
    use crate::web;

    fn probe(body_class: &str, hrefs: &[&str]) -> web::Probe {
        web::Probe {
            url: "/".to_string(),
            hash: String::new(),
            origin: "http://127.0.0.1:1".to_string(),
            body_class: body_class.to_string(),
            scrolled: false,
            scrollable: false,
            typed: false,
            elements: hrefs
                .iter()
                .enumerate()
                .map(|(index, href)| web::ProbeElement {
                    index,
                    group: "ul>li>a".to_string(),
                    tag: "a".to_string(),
                    href: Some((*href).to_string()),
                    absolute: None,
                    input_type: None,
                    label: String::new(),
                })
                .collect(),
        }
    }

    #[test]
    fn test_usage() {
        let patterns = vec!["/p/{id}/file/{*path}".to_string()];
        let a = fingerprint("/", &probe("", &["/p/x/file/a.rs"]), &patterns);
        let b = fingerprint(
            "/",
            &probe("", &["/p/x/file/a.rs", "/p/x/file/b.rs"]),
            &patterns,
        );
        let c = fingerprint("/", &probe("drawer-open", &["/p/x/file/a.rs"]), &patterns);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
