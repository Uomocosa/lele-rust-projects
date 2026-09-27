use crate::project;

pub fn changes_since(
    registry: &project::Registry,
    id: &str,
    since: u64,
) -> Option<project::ChangeFeed> {
    let live = registry.live.lock().ok()?;
    let entry = live.projects.get(id)?;
    let mut paths: Vec<String> = entry
        .changes
        .iter()
        .filter(|(_, change)| change.version > since)
        .map(|(rel, _)| rel.to_string_lossy().to_string())
        .collect();
    let feed = project::ChangeFeed {
        version: entry.version,
        reset: since < entry.base_version,
        paths: Vec::new(),
    };
    drop(live);
    paths.sort();
    Some(project::ChangeFeed { paths, ..feed })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Instant;

    use super::changes_since;
    use crate::project;

    #[test]
    fn test_usage() {
        let registry = project::Registry::default();
        let (notify, _) = tokio::sync::watch::channel(5);
        let mut changes = HashMap::new();
        for (path, version) in [("src/a.rs", 4), ("src/b.rs", 6)] {
            changes.insert(
                PathBuf::from(path),
                project::FileChange {
                    version,
                    at: Instant::now(),
                    lines: Vec::new(),
                    text: None,
                },
            );
        }
        registry.live.lock().unwrap().projects.insert(
            "demo".to_string(),
            project::LiveProject {
                root: PathBuf::from("/x"),
                base_version: 3,
                version: 6,
                last_access: Instant::now(),
                dirs: Vec::new(),
                changes,
                notify,
            },
        );
        let feed = changes_since(&registry, "demo", 5).unwrap();
        assert_eq!(feed.paths, vec!["src/b.rs".to_string()]);
        assert!(!feed.reset);
        assert!(changes_since(&registry, "demo", 1).unwrap().reset);
        assert!(changes_since(&registry, "other", 0).is_none());
    }
}
