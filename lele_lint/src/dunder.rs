use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Dunder {
    #[serde(default)]
    pub folders: HashMap<String, String>,
    #[serde(default)]
    pub files: Vec<String>,
}

impl Default for Dunder {
    fn default() -> Self {
        let mut folders = HashMap::new();
        folders.insert("__basic__".to_string(), "basic".to_string());
        Self {
            folders,
            files: Vec::new(),
        }
    }
}

// no test_usage necessary
