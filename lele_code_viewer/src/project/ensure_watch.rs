use std::collections::HashMap;
use std::time::Instant;

use crate::project;

pub fn ensure_watch(registry: &project::Registry, item: &project::ProjectRef) -> u64 {
    let Ok(mut live) = registry.live.lock() else {
        return 0;
    };
    if let Some(entry) = live.projects.get_mut(&item.id) {
        entry.last_access = Instant::now();
        return entry.version;
    }
    if live.watcher.is_none() {
        return 0;
    }
    live.next_version = live.next_version.saturating_add(1);
    let version = live.next_version;
    let (notify, _) = tokio::sync::watch::channel(version);
    live.projects.insert(
        item.id.clone(),
        project::LiveProject {
            root: item.root.clone(),
            base_version: version,
            version,
            last_access: Instant::now(),
            dirs: Vec::new(),
            changes: HashMap::new(),
            notify,
        },
    );
    project::add_watch_dirs(&mut live, &item.id, project::watch_dirs(&item.root));
    if let Ok(mut cache) = registry.cache.write() {
        cache.remove(&item.id);
    }
    version
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::ensure_watch;
    use crate::project;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let item = project::ProjectRef {
            id: "demo".to_string(),
            name: "demo".to_string(),
            root: dir.path().to_path_buf(),
        };
        let unwatched = project::Registry::default();
        assert_eq!(ensure_watch(&unwatched, &item), 0);

        let registry = project::Registry {
            watch: true,
            live: Mutex::new(project::LiveState {
                watcher: Some(notify::recommended_watcher(|_| {}).unwrap()),
                ..project::LiveState::default()
            }),
            ..project::Registry::default()
        };
        let first = ensure_watch(&registry, &item);
        assert!(first > 0);
        assert_eq!(ensure_watch(&registry, &item), first);
        assert_eq!(registry.live.lock().unwrap().projects["demo"].dirs.len(), 1);
    }
}
