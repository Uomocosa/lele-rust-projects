use std::collections::HashMap;

use crate::render::basic::enums;

pub struct LinkConfig {
    pub prefix: String,
    pub assets: String,
    pub html: bool,
    pub nav: Option<Nav>,
}

pub struct Nav {
    pub current: String,
    pub name: String,
    pub root: String,
    pub projects: Vec<NavProject>,
    pub view: enums::ViewKind,
    pub live: Option<Live>,
}

pub struct Live {
    pub events: String,
    pub version: u64,
    pub watch: String,
    pub recent: HashMap<String, Vec<usize>>,
}

pub struct NavProject {
    pub id: String,
    pub name: String,
}

// no test_usage necessary
