use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

#[derive(Default, Deserialize)]
struct CargoManifest {
    #[serde(default)]
    dependencies: HashMap<String, toml::Value>,
    #[serde(default, rename = "dev-dependencies")]
    dev_dependencies: HashMap<String, toml::Value>,
    #[serde(default, rename = "build-dependencies")]
    build_dependencies: HashMap<String, toml::Value>,
    #[serde(default)]
    target: HashMap<String, TargetSection>,
}

#[derive(Default, Deserialize)]
struct TargetSection {
    #[serde(default)]
    dependencies: HashMap<String, toml::Value>,
}

fn manifest_has_freenet(manifest: &CargoManifest) -> bool {
    let uses_freenet = |name: &str| name.to_lowercase().contains("freenet");
    manifest.dependencies.keys().any(|d| uses_freenet(d))
        || manifest.dev_dependencies.keys().any(|d| uses_freenet(d))
        || manifest.build_dependencies.keys().any(|d| uses_freenet(d))
        || manifest
            .target
            .values()
            .flat_map(|t| t.dependencies.keys())
            .any(|d| uses_freenet(d))
}

pub fn uses_freenet(crate_path: &Path) -> bool {
    if crate_path.join("contract").join("Cargo.toml").exists() {
        return true;
    }
    let cargo = crate_path.join("Cargo.toml");
    let content = match std::fs::read_to_string(&cargo) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let Ok(manifest) = toml::from_str::<CargoManifest>(&content) else {
        return content.to_lowercase().contains("freenet");
    };
    manifest_has_freenet(&manifest)
}

#[cfg(test)]
mod tests {
    use super::uses_freenet;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"foo\"\n").unwrap();
        assert!(!uses_freenet(dir.path()));
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[dependencies]\nfreenet = \"0.1\"\n",
        )
        .unwrap();
        assert!(uses_freenet(dir.path()));
    }

    #[test]
    fn test_contract_detection() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
        std::fs::create_dir_all(dir.path().join("contract")).unwrap();
        std::fs::write(
            dir.path().join("contract/Cargo.toml"),
            "[package]\nname=\"c\"\n",
        )
        .unwrap();
        assert!(uses_freenet(dir.path()));
    }
}
