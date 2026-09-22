use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Default)]
pub struct LeleLintSection {
    #[serde(default)]
    pub checkers: HashMap<String, bool>,
    #[serde(default)]
    pub layout: Option<String>,
}
