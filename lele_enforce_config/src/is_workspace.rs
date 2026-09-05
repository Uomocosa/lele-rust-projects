use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    #[serde(default)]
    workspace: Option<WorkspaceSection>,
}

#[derive(Deserialize)]
struct WorkspaceSection {}

pub fn is_workspace(crate_path: &Path) -> bool {
    let cargo = crate_path.join("Cargo.toml");
    let content = match std::fs::read_to_string(&cargo) {
        Ok(c) => c,
        Err(_) => return false,
    };
    if let Ok(manifest) = toml::from_str::<Manifest>(&content) {
        if manifest.workspace.is_some() {
            return true;
        }
    }
    content
        .lines()
        .any(|l| l.trim_start().starts_with("[workspace"))
}

#[cfg(test)]
mod tests {
    use super::is_workspace;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let crate_a = dir.path().join("my_crate");
        std::fs::create_dir_all(&crate_a).unwrap();
        std::fs::write(crate_a.join("Cargo.toml"), "[package]\nname=\"my_crate\"\n").unwrap();
        let ws = dir.path().join("ws_crate");
        std::fs::create_dir_all(&ws).unwrap();
        std::fs::write(ws.join("Cargo.toml"), "[workspace]\nmembers=[]\n").unwrap();
        assert!(!is_workspace(&crate_a));
        assert!(is_workspace(&ws));
    }
}
