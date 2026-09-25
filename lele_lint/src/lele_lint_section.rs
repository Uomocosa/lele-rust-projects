use serde::Deserialize;
use std::collections::HashMap;

use crate::Dunder;

#[derive(Deserialize, Debug, Default)]
pub struct LeleLintSection {
    #[serde(default)]
    pub checkers: HashMap<String, bool>,
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub dunder: Dunder,
}

// no test_usage necessary
