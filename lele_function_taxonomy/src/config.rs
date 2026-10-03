use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("TAX002: {0}: {1}")]
    Invalid(String, String),
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LeleToml {
    #[serde(default)]
    pub lele: LeleSection,
    #[serde(default)]
    pub honesty: HonestySection,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LeleSection {
    #[serde(default)]
    pub boundary: Vec<BoundaryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryEntry {
    pub name: String,
    pub why: String,
    pub folders: Vec<String>,
    #[serde(default)]
    pub cannot_use: Vec<String>,
    #[serde(default)]
    pub require: Option<Requirement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Requirement {
    Honest,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HonestySection {
    #[serde(default)]
    pub declared_honest: Vec<String>,
    #[serde(default)]
    pub declared_dishonest: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub boundaries: Vec<BoundaryEntry>,
    pub declared_honest: Vec<String>,
    pub declared_dishonest: Vec<String>,
}

impl Config {
    pub fn honest_boundaries(&self) -> impl Iterator<Item = &BoundaryEntry> {
        self.boundaries
            .iter()
            .filter(|b| b.require == Some(Requirement::Honest))
    }

    pub fn has_honest_boundary(&self) -> bool {
        self.honest_boundaries().next().is_some()
    }
}

// needed helper: keep only the actionable first line of a toml error
fn first_line(message: &str) -> String {
    message
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or(message)
        .trim()
        .to_string()
}

pub fn lele_toml_path(manifest_path: &Path) -> PathBuf {
    manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("lele.toml")
}

// needed helper: portable display path for diagnostics ("lele.toml" not absolute)
pub fn config_display_path(manifest_path: &Path) -> String {
    manifest_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map_or_else(
            || "lele.toml".to_string(),
            |name| format!("{name}/lele.toml"),
        )
}

pub fn load(manifest_path: &Path) -> Result<Config, ConfigError> {
    let path = lele_toml_path(manifest_path);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Ok(Config::default());
    };
    let display = config_display_path(manifest_path);
    let parsed: LeleToml = toml::from_str(&content)
        .map_err(|e| ConfigError::Invalid(display, first_line(&e.to_string())))?;
    Ok(Config {
        boundaries: parsed.lele.boundary,
        declared_honest: parsed.honesty.declared_honest,
        declared_dishonest: parsed.honesty.declared_dishonest,
    })
}

pub fn validate(manifest_path: &Path, config: &Config) -> Result<(), ConfigError> {
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let display = config_display_path(manifest_path);
    for boundary in &config.boundaries {
        if boundary.folders.is_empty() {
            return Err(ConfigError::Invalid(
                display.clone(),
                format!("boundary `{}` has no folders", boundary.name),
            ));
        }
        for folder in &boundary.folders {
            if !root.join(folder).is_dir() {
                return Err(ConfigError::Invalid(
                    display.clone(),
                    format!(
                        "boundary `{}` names folder `{folder}` which does not exist",
                        boundary.name
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub fn is_inside_boundary(rel: &Path, folders: &[String]) -> bool {
    folders.iter().any(|folder| rel.starts_with(folder))
}

#[cfg(test)]
mod tests {
    use super::{is_inside_boundary, load};
    use std::path::Path;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = tmp.path().join("Cargo.toml");
        std::fs::write(&manifest, "").unwrap();
        std::fs::write(
            tmp.path().join("lele.toml"),
            "[[lele.boundary]]\nname = \"b\"\nwhy = \"w\"\nfolders = [\"src/core\"]\nrequire = \"honest\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join("src/core")).unwrap();
        let config = load(&manifest).unwrap();
        assert!(config.has_honest_boundary());
        assert!(is_inside_boundary(
            Path::new("src/core/x.rs"),
            &["src/core".to_string()]
        ));
        assert!(!is_inside_boundary(
            Path::new("src/other/x.rs"),
            &["src/core".to_string()]
        ));
    }

    #[test]
    fn test_usage_missing_folder_fails_validation() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = tmp.path().join("Cargo.toml");
        std::fs::write(&manifest, "").unwrap();
        std::fs::write(
            tmp.path().join("lele.toml"),
            "[[lele.boundary]]\nname = \"b\"\nwhy = \"w\"\nfolders = [\"src/nope\"]\nrequire = \"honest\"\n",
        )
        .unwrap();
        let config = load(&manifest).unwrap();
        assert!(super::validate(&manifest, &config).is_err());
    }

    #[test]
    fn test_usage_removed_key_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = tmp.path().join("Cargo.toml");
        std::fs::write(&manifest, "").unwrap();
        std::fs::write(
            tmp.path().join("lele.toml"),
            "[honesty]\nhonesty_depth = 1\n",
        )
        .unwrap();
        assert!(load(&manifest).is_err());
    }
}
