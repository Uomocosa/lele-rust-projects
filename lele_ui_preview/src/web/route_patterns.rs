use std::path::Path;

pub fn route_patterns(crate_dir: &Path, files: &[String]) -> Vec<String> {
    let mut patterns: Vec<String> = Vec::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(crate_dir.join(file)) else {
            continue;
        };
        for chunk in text.split(".route(").skip(1) {
            let Some(literal) = leading_string(chunk) else {
                continue;
            };
            if !patterns.contains(&literal) {
                patterns.push(literal);
            }
        }
    }
    patterns
}

// needed helper: the string literal that opens a chunk, if any
fn leading_string(chunk: &str) -> Option<String> {
    let rest = chunk.trim_start().strip_prefix('"')?;
    let end = rest.find('"')?;
    rest.get(..end).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::route_patterns;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("router.rs"),
            "Router::new()\n    .route(\"/\", get(home))\n    .route(\"/p/{id}\", get(p))\n    .route(\n        \"/p/{id}/file/{*path}\", get(f))\n    .route(\"/\", get(dup))",
        )
        .unwrap();
        let patterns = route_patterns(dir.path(), &["router.rs".to_string()]);
        assert_eq!(patterns, vec!["/", "/p/{id}", "/p/{id}/file/{*path}"]);
    }
}
