use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::project;
use crate::server;

pub fn rescan(state: &Arc<server::AppState>) -> bool {
    if state.scanning.swap(true, Ordering::SeqCst) {
        return false;
    }
    let state = Arc::clone(state);
    tokio::task::spawn_blocking(move || {
        let rules = match state.settings.read() {
            Ok(settings) => settings.ignore.clone(),
            Err(poisoned) => poisoned.into_inner().ignore.clone(),
        };
        let compiled = project::compile_rules(&rules).unwrap_or_default();
        let found = project::discover_projects(&state.registry.roots, &compiled);
        println!("lele_code_viewer: rescan found {} projects", found.len());
        match state.projects.write() {
            Ok(mut list) => *list = Arc::new(found),
            Err(poisoned) => *poisoned.into_inner() = Arc::new(found),
        }
        state.scanning.store(false, Ordering::SeqCst);
    });
    true
}

// no test_usage necessary
