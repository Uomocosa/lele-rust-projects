use std::path::Path;

use serde::Deserialize;

use derive_more::Deref;

use super::config_checker_enabled;
use super::config_load;
use crate::error;
use crate::lele_lint_section;

pub const CONFIG_FILENAME: &str = "lele_lint.toml";

#[derive(Deserialize, Debug, Default, Deref)]
pub struct Config(pub Option<lele_lint_section::LeleLintSection>);

#[rustfmt::skip]
impl Config {
    pub fn load(project_root: &Path) -> Result<Self, error::Error> {
        config_load::load(project_root)
    }

    pub fn checker_enabled(&self, name: &str) -> bool {
        config_checker_enabled::checker_enabled(self, name)
    }
}

// no test_usage necessary
