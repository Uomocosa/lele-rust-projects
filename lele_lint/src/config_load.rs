// no test_usage necessary
use std::path::Path;

use super::config::Config;
use crate::error::Error;

pub(crate) fn load(project_root: &Path) -> Result<Config, Error> {
    let config_path = project_root.join("lele_lint.toml");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&content)?)
    } else {
        Ok(Config::default())
    }
}
