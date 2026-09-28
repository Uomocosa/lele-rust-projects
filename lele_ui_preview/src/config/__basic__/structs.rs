use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LeleToml {
    pub ui_preview: Option<PreviewConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PreviewConfig {
    pub web: Option<WebConfig>,
    pub bevy: Option<BevyConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Viewport {
    pub name: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebConfig {
    pub command: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub fixture: Option<String>,
    pub start: Option<String>,
    #[serde(default)]
    pub routes_from: Vec<String>,
    #[serde(default)]
    pub keys_from: Vec<String>,
    #[serde(default)]
    pub skip_paths: Vec<String>,
    pub fill_text: Option<String>,
    pub viewports: Vec<Viewport>,
    pub max_states: Option<usize>,
    pub max_depth: Option<usize>,
    pub settle_ms: Option<u64>,
    pub startup_timeout_secs: Option<u64>,
    pub chrome: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BevyConfig {
    pub command: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub resource: String,
    pub max_states: Option<usize>,
    pub settle_ms: Option<u64>,
    pub startup_timeout_secs: Option<u64>,
}
