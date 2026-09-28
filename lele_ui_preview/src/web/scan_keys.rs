use std::path::Path;

pub fn scan_keys(crate_dir: &Path, files: &[String]) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(crate_dir.join(file)) else {
            continue;
        };
        for chunk in text.split(".key").skip(1) {
            let Some(key) = compared_literal(chunk) else {
                continue;
            };
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
    }
    keys
}

// needed helper: the quoted literal in `=== "X"` / `== 'X'` right after `.key`
fn compared_literal(chunk: &str) -> Option<String> {
    let rest = chunk.trim_start();
    let rest = rest
        .strip_prefix("===")
        .or_else(|| rest.strip_prefix("=="))?
        .trim_start();
    let quote = rest.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let body = rest.get(1..)?;
    let end = body.find(quote)?;
    body.get(..end)
        .filter(|k| !k.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::scan_keys;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("app.js"),
            "if (event.key === \"Escape\") {}\nif (e.key == 'k') {}\nconst key = e.keyCode;\nif (event.key === \"Escape\") {}",
        )
        .unwrap();
        assert_eq!(
            scan_keys(dir.path(), &["app.js".to_string()]),
            vec!["Escape", "k"]
        );
    }
}
