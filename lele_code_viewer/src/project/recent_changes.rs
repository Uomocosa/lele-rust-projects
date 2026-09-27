use std::collections::HashMap;

use crate::project;

pub fn recent_changes(registry: &project::Registry, id: &str) -> HashMap<String, Vec<usize>> {
    let Ok(live) = registry.live.lock() else {
        return HashMap::new();
    };
    let Some(entry) = live.projects.get(id) else {
        return HashMap::new();
    };
    entry
        .changes
        .iter()
        .filter(|(_, change)| change.at.elapsed() < project::RECENT_WINDOW)
        .map(|(rel, change)| (rel.to_string_lossy().to_string(), change.lines.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Instant;

    use super::recent_changes;
    use crate::project;

    #[test]
    fn test_usage() {
        let registry = project::Registry::default();
        assert!(recent_changes(&registry, "demo").is_empty());
        let (notify, _) = tokio::sync::watch::channel(2);
        let mut changes = HashMap::new();
        changes.insert(
            PathBuf::from("src/a.rs"),
            project::FileChange {
                version: 2,
                at: Instant::now(),
                lines: vec![3],
                text: None,
            },
        );
        registry.live.lock().unwrap().projects.insert(
            "demo".to_string(),
            project::LiveProject {
                root: PathBuf::from("/x"),
                base_version: 1,
                version: 2,
                last_access: Instant::now(),
                dirs: Vec::new(),
                changes,
                notify,
            },
        );
        let recent = recent_changes(&registry, "demo");
        assert_eq!(recent.get("src/a.rs"), Some(&vec![3]));
    }
}
