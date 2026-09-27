use std::sync::Arc;

use crate::Error;
use crate::project;

pub fn start_watcher(registry: &Arc<project::Registry>) -> Result<(), Error> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            let _ = tx.send(event);
        }
    })?;
    if let Ok(mut live) = registry.live.lock() {
        live.watcher = Some(watcher);
    }
    tokio::spawn(project::watch_loop(registry.clone(), rx));
    let sweeper = registry.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(project::SWEEP_EVERY);
        loop {
            tick.tick().await;
            project::sweep_live(&sweeper, project::IDLE_UNWATCH);
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::start_watcher;
    use crate::project;

    #[tokio::test]
    async fn test_usage() {
        let registry = Arc::new(project::Registry::default());
        start_watcher(&registry).unwrap();
        assert!(registry.live.lock().unwrap().watcher.is_some());
    }
}
