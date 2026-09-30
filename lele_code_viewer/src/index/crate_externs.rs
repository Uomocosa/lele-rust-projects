use std::collections::BTreeSet;
use std::path::Path;

pub fn crate_externs(root: &Path) -> BTreeSet<String> {
    let Ok(text) = std::fs::read_to_string(root.join("Cargo.toml")) else {
        return BTreeSet::new();
    };
    let mut out = BTreeSet::new();
    let mut in_deps = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps = matches!(
                trimmed,
                "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
            );
            continue;
        }
        if !in_deps || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, _)) = trimmed.split_once('=') else {
            continue;
        };
        let name = key
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .replace('-', "_");
        if !name.is_empty() {
            out.insert(name);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::crate_externs;
    use crate::index;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.0.0\"\n\n[dependencies]\nproc-macro2 = \"1\"\nserde = \"1\"\n\n[dev-dependencies]\ntempfile = \"3\"\n",
        )
        .unwrap();
        let externs = crate_externs(dir.path());
        assert!(externs.contains("proc_macro2"));
        assert!(!externs.contains(index::crate_name_of(dir.path()).as_str()));
        assert!(externs.contains("serde"));
        assert!(externs.contains("tempfile"));
        assert!(crate_externs(std::path::Path::new("/nonexistent")).is_empty());
    }
}
