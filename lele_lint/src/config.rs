use std::path::Path;

use atomic_delegate_macros::atomic_delegates;
use serde::Deserialize;

use derive_more::Deref;

use crate::Dunder;
use crate::Error;
use crate::LeleTomlLintSections;

pub const CONFIG_FILENAME: &str = "lele.toml";

#[derive(Deserialize, Debug, Default, Deref)]
pub struct Config(pub Option<LeleTomlLintSections>);

#[atomic_delegates]
impl Config {
    pub fn load(project_root: &Path) -> Result<Self, Error> {}
    pub fn dunder(&self) -> Dunder {}
}

// no test_usage necessary
