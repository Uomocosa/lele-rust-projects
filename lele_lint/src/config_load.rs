use std::path::Path;

use serde::Deserialize;

use super::config::Config;
use crate::Error;
use crate::LeleLintSection;

#[derive(Deserialize)]
struct LeleToml {
    #[serde(default)]
    lele: LeleSection,
}

#[derive(Default, Deserialize)]
struct LeleSection {
    #[serde(default)]
    lint: LeleLintSection,
}

pub(crate) fn load(project_root: &Path) -> Result<Config, Error> {
    let config_path = project_root.join("lele.toml");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Ok(Config::default()),
    };
    let parsed: LeleToml = toml::from_str(&content)?;
    Ok(Config(Some(parsed.lele.lint)))
}

// no test_usage necessary
