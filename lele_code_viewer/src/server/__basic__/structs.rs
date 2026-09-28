use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;

use crate::project;
use crate::source;

pub struct AppState {
    pub registry: Arc<project::Registry>,
    pub projects: RwLock<Arc<Vec<project::ProjectRef>>>,
    pub hl: Arc<source::Highlighter>,
    pub settings: RwLock<project::Settings>,
    pub settings_path: PathBuf,
    pub self_update: bool,
    pub scanning: AtomicBool,
}

pub struct ServeOptions {
    pub settings_path: PathBuf,
    pub self_update: bool,
}

// no test_usage necessary
