use std::path::PathBuf;

use notify::Watcher;

use crate::project;

pub fn add_watch_dirs(live: &mut project::LiveState, id: &str, dirs: Vec<PathBuf>) {
    let project::LiveState {
        watcher,
        dir_refs,
        projects,
        ..
    } = live;
    let Some(watcher) = watcher.as_mut() else {
        return;
    };
    let Some(entry) = projects.get_mut(id) else {
        return;
    };
    for dir in dirs {
        if entry.dirs.contains(&dir) || entry.dirs.len() >= project::MAX_WATCH_DIRS {
            continue;
        }
        let count = dir_refs.get(&dir).copied().unwrap_or(0);
        if count == 0
            && watcher
                .watch(&dir, notify::RecursiveMode::NonRecursive)
                .is_err()
        {
            continue;
        }
        dir_refs.insert(dir.clone(), count.saturating_add(1));
        entry.dirs.push(dir);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Instant;

    use super::add_watch_dirs;
    use crate::project;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let mut live = project::LiveState {
            watcher: Some(notify::recommended_watcher(|_| {}).unwrap()),
            ..project::LiveState::default()
        };
        let (notify, _) = tokio::sync::watch::channel(1);
        live.projects.insert(
            "demo".to_string(),
            project::LiveProject {
                root: dir.path().to_path_buf(),
                base_version: 1,
                version: 1,
                last_access: Instant::now(),
                dirs: Vec::new(),
                changes: HashMap::new(),
                notify,
            },
        );
        add_watch_dirs(&mut live, "demo", vec![dir.path().to_path_buf()]);
        add_watch_dirs(&mut live, "demo", vec![dir.path().to_path_buf()]);
        assert_eq!(live.dir_refs.get(dir.path()), Some(&1));
        assert_eq!(live.projects["demo"].dirs.len(), 1);
    }
}
