use std::time::Duration;

use notify::Watcher;

use crate::project;

pub fn sweep_live(registry: &project::Registry, idle: Duration) {
    let Ok(mut live) = registry.live.lock() else {
        return;
    };
    let project::LiveState {
        watcher,
        dir_refs,
        projects,
        ..
    } = &mut *live;
    let idle_ids: Vec<String> = projects
        .iter()
        .filter(|(_, entry)| {
            entry.notify.receiver_count() == 0 && entry.last_access.elapsed() > idle
        })
        .map(|(id, _)| id.clone())
        .collect();
    for id in idle_ids {
        let Some(entry) = projects.remove(&id) else {
            continue;
        };
        for dir in entry.dirs {
            let count = dir_refs.get(&dir).copied().unwrap_or(0).saturating_sub(1);
            if count > 0 {
                dir_refs.insert(dir, count);
                continue;
            }
            dir_refs.remove(&dir);
            if let Some(watcher) = watcher.as_mut() {
                let _ = watcher.unwatch(&dir);
            }
        }
        if let Ok(mut cache) = registry.cache.write() {
            cache.remove(&id);
        }
    }
    for entry in projects.values_mut() {
        for change in entry.changes.values_mut() {
            if change.at.elapsed() > project::RECENT_WINDOW {
                change.text = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;

    use super::sweep_live;
    use crate::index;
    use crate::project;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let registry = project::Registry {
            watch: true,
            live: Mutex::new(project::LiveState {
                watcher: Some(notify::recommended_watcher(|_| {}).unwrap()),
                ..project::LiveState::default()
            }),
            ..project::Registry::default()
        };
        let item = project::ProjectRef {
            id: "demo".to_string(),
            name: "demo".to_string(),
            root: dir.path().to_path_buf(),
        };
        project::ensure_watch(&registry, &item);
        registry
            .cache
            .write()
            .unwrap()
            .insert("demo".to_string(), Arc::new(index::SymbolIndex::default()));

        let subscriber = registry.live.lock().unwrap().projects["demo"]
            .notify
            .subscribe();
        sweep_live(&registry, Duration::ZERO);
        assert!(registry.live.lock().unwrap().projects.contains_key("demo"));

        drop(subscriber);
        sweep_live(&registry, Duration::ZERO);
        let emptied = registry
            .live
            .lock()
            .map(|live| live.projects.is_empty() && live.dir_refs.is_empty())
            .unwrap();
        assert!(emptied);
        assert!(registry.cache.read().unwrap().is_empty());
    }
}
