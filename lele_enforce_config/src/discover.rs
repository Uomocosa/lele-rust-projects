use std::path::Path;
use std::path::PathBuf;

use crate::LeleConfig;
use crate::is_workspace;

pub fn discover(root: &Path, exclude: &LeleConfig) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if exclude.is_excluded(root, &path) {
            continue;
        }
        if !path.is_dir() {
            continue;
        }
        if name.starts_with('.') {
            continue;
        }
        if !path.join("Cargo.toml").exists() {
            continue;
        }
        if is_workspace(&path) {
            continue;
        }
        out.push(path);
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::discover;
    use crate::ExcludedPath;
    use crate::LeleConfig;

    fn config_with(names: &[&str]) -> LeleConfig {
        LeleConfig {
            excluded_paths: names
                .iter()
                .map(|n| ExcludedPath((*n).to_string()))
                .collect(),
        }
    }

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let crate_a = dir.path().join("my_crate");
        std::fs::create_dir_all(&crate_a).unwrap();
        std::fs::write(crate_a.join("Cargo.toml"), "[package]\nname=\"my_crate\"\n").unwrap();
        let ws = dir.path().join("ws_crate");
        std::fs::create_dir_all(&ws).unwrap();
        std::fs::write(ws.join("Cargo.toml"), "[workspace]\nmembers=[]\n").unwrap();
        let found = discover(dir.path(), &LeleConfig::default());
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("my_crate"));
        let found2 = discover(dir.path(), &config_with(&["my_crate"]));
        assert_eq!(found2, Vec::<std::path::PathBuf>::new());
    }
}
