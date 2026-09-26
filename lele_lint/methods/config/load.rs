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
struct LeleSection {
    #[serde(default)]
    lint: LeleTomlLintSections,
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
}
