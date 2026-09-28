pub fn match_route(patterns: &[String], path: &str) -> Option<String> {
    patterns
        .iter()
        .find(|pattern| matches(pattern, path))
        .cloned()
}

// needed helper: axum-style `{name}` / `{*rest}` segment matching
fn matches(pattern: &str, path: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.trim_matches('/').split('/').collect();
    let path_parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    let mut remaining = path_parts.iter();
    for part in &pattern_parts {
        if part.starts_with("{*") {
            return remaining.next().is_some_and(|segment| !segment.is_empty());
        }
        let Some(segment) = remaining.next() else {
            return false;
        };
        let is_param = part.starts_with('{') && part.ends_with('}');
        if is_param && segment.is_empty() {
            return false;
        }
        if !is_param && part != segment {
            return false;
        }
    }
    remaining.next().is_none()
}

#[cfg(test)]
mod tests {
    use super::match_route;

    #[test]
    fn test_usage() {
        let patterns: Vec<String> = ["/", "/p/{id}", "/p/{id}/file/{*path}", "/p/{id}/search"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert_eq!(match_route(&patterns, "/").as_deref(), Some("/"));
        assert_eq!(
            match_route(&patterns, "/p/clicker").as_deref(),
            Some("/p/{id}")
        );
        assert_eq!(
            match_route(&patterns, "/p/clicker/").as_deref(),
            Some("/p/{id}")
        );
        assert_eq!(
            match_route(&patterns, "/p/clicker/file/src/lib.rs").as_deref(),
            Some("/p/{id}/file/{*path}")
        );
        assert_eq!(
            match_route(&patterns, "/p/clicker/search").as_deref(),
            Some("/p/{id}/search")
        );
        assert_eq!(match_route(&patterns, "/assets/app.js"), None);
    }
}
