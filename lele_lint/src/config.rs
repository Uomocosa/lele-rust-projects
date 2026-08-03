// lele_lint: allow E001
// no test_usage necessary
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::error::Error;

const CONFIG_FILENAME: &str = "lele_lint.toml";

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub lele_lint: Option<LeleLintSection>,
}

#[derive(Deserialize, Debug, Default)]
pub struct LeleLintSection {
    #[serde(default)]
    pub bevy_mode: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub checkers: HashMap<String, bool>,
}

impl Config {
    pub fn load(project_root: &Path) -> Result<Self, Error> {
        let config_path = project_root.join(CONFIG_FILENAME);
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Config::default())
        }
    }

    #[allow(dead_code)]
    pub fn bevy_mode(&self) -> bool {
        self.lele_lint
            .as_ref()
            .map(|s| s.bevy_mode)
            .unwrap_or(false)
    }

    pub fn checker_enabled(&self, name: &str) -> bool {
        self.lele_lint
            .as_ref()
            .and_then(|s| s.checkers.get(name))
            .copied()
            .unwrap_or(true)
    }
}
