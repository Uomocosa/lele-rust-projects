use std::path::Path;

use serde::Deserialize;

use crate::Config;
use crate::Error;
use crate::LeleTomlLintSections;

#[derive(Deserialize)]
struct LeleToml {
    #[serde(default)]
    lele: LeleSection,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct LeleSection {
    #[serde(default)]
    lint: LeleTomlLintSections,
    #[serde(default, rename = "config")]
    _config: Option<toml::Value>,
}

pub fn load(project_root: &Path) -> Result<Config, Error> {
    let config_path = project_root.join("lele.toml");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Ok(Config::default()),
    };
    let parsed: LeleToml = toml::from_str(&content)?;
    Ok(Config(Some(parsed.lele.lint)))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::load;

    #[test]
    fn test_usage() {
        let config = load(Path::new("/nonexistent-lele-lint-probe")).unwrap();
        assert!(config.as_ref().is_none());
    }

    #[test]
    fn test_usage_rejects_unknown_lele_keys() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("lele.toml"),
            "[lele.lint.checkers]\ncontainer_placement = false\n",
        )
        .unwrap();
        let err = load(dir.path()).unwrap_err().to_string();
        assert!(err.contains("checkers"), "{err}");
    }

    #[test]
    fn test_usage_ignores_other_tools_sections() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("lele.toml"),
            "[honesty]\nhonesty_depth = 1\n\n[ui_preview.bevy]\nresource = \"Snapshot\"\n\n[lele.config]\nexclude = [\"target\"]\n\n[lele.lint]\n",
        )
        .unwrap();
        assert!(load(dir.path()).unwrap().as_ref().is_some());
    }
}
