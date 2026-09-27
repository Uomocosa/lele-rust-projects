use std::sync::Arc;

use crate::project;
use crate::source;

pub struct AppState {
    pub registry: Arc<project::Registry>,
    pub projects: Arc<Vec<project::ProjectRef>>,
    pub hl: Arc<source::Highlighter>,
}

// no test_usage necessary
