use serde::Deserialize;

use crate::web;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ProbeElement {
    pub index: usize,
    pub group: String,
    pub tag: String,
    pub href: Option<String>,
    pub absolute: Option<String>,
    pub input_type: Option<String>,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Probe {
    pub url: String,
    pub hash: String,
    pub origin: String,
    pub body_class: String,
    pub scrolled: bool,
    pub scrollable: bool,
    pub typed: bool,
    pub elements: Vec<ProbeElement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebState {
    pub location: String,
    pub steps: Vec<web::Action>,
    pub path: Vec<String>,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub base_url: String,
    pub patterns: Vec<String>,
    pub keys: Vec<String>,
    pub skip_paths: Vec<String>,
    pub fill_text: String,
    pub settle_ms: u64,
    pub max_states: usize,
    pub max_depth: usize,
}
