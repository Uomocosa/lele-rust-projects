use crate::web;

pub fn element_group(element: &web::ProbeElement, patterns: &[String]) -> String {
    let target = element.href.as_deref().map_or_else(String::new, |href| {
        if href.starts_with('#') {
            "#".to_string()
        } else {
            let path = href.split(['?', '#']).next().unwrap_or(href);
            web::match_route(patterns, path).unwrap_or_else(|| "other".to_string())
        }
    });
    format!("{}->{target}", element.group)
}

#[cfg(test)]
mod tests {
    use super::element_group;
    use crate::web;

    #[test]
    fn test_usage() {
        let element = web::ProbeElement {
            index: 0,
            group: "nav>a".to_string(),
            tag: "a".to_string(),
            href: Some("/p/x/file/a.rs#L3".to_string()),
            absolute: None,
            input_type: None,
            label: String::new(),
        };
        let patterns = vec!["/p/{id}/file/{*path}".to_string()];
        assert_eq!(
            element_group(&element, &patterns),
            "nav>a->/p/{id}/file/{*path}"
        );
    }
}
