use std::path::Path;

pub fn crate_name_of(root: &Path) -> String {
    let Ok(text) = std::fs::read_to_string(root.join("Cargo.toml")) else {
        return "crate".to_string();
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("name")
            && let Some(rest) = rest.trim_start().strip_prefix('=')
        {
            let value = rest.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return value.to_string();
            }
        }
    }
    "crate".to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::crate_name_of;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.0.0\"\n",
        )
        .unwrap();
        assert_eq!(crate_name_of(dir.path()), "demo");
        assert_eq!(crate_name_of(std::path::Path::new("/nonexistent")), "crate");
    }
}
