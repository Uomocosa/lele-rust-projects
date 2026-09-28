use std::sync::Arc;

use crate::project;
use crate::server;

pub fn project_list(state: &server::AppState) -> Arc<Vec<project::ProjectRef>> {
    match state.projects.read() {
        Ok(list) => Arc::clone(&list),
        Err(poisoned) => Arc::clone(&poisoned.into_inner()),
    }
}

// no test_usage necessary
