use std::path::Path;

use serde::Deserialize;

use derive_more::Deref;

use super::config_checker_enabled;
use super::config_dunder;
use super::config_layout;
use super::config_load;
use crate::Dunder;
use crate::Error;
use crate::Layout;
use crate::LeleLintSection;

pub const CONFIG_FILENAME: &str = "lele.toml";

#[derive(Deserialize, Debug, Default, Deref)]
pub struct Config(pub Option<LeleLintSection>);

#[rustfmt::skip]
impl Config {
    pub fn load(project_root: &Path) -> Result<Self, Error> {
        config_load::load(project_root)
    }

    pub fn checker_enabled(&self, name: &str) -> bool {
        config_checker_enabled::checker_enabled(self, name)
    }

    pub fn layout(&self) -> Layout {
        config_layout::layout(self)
    }

    pub fn dunder(&self) -> Dunder {
        config_dunder::dunder(self)
    }
}

// no test_usage necessary
