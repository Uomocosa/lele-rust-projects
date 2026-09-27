use std::collections::HashSet;
use std::hash::BuildHasher;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use crate::project;

pub fn record_changes<S: BuildHasher>(registry: &project::Registry, paths: &HashSet<PathBuf, S>) {
    let Ok(mut live) = registry.live.lock() else {
        return;
    };
    let roots: Vec<(String, PathBuf)> = live
        .projects
        .iter()
        .map(|(id, entry)| (id.clone(), entry.root.clone()))
        .collect();
    for (id, root) in roots {
        let mut rels: Vec<PathBuf> = paths
            .iter()
            .filter_map(|p| relevant_rel(&root, p))
            .collect();
        if rels.is_empty() {
            continue;
        }
        rels.sort();
        for rel in &rels {
            let full = root.join(rel);
            if full.is_dir() {
                project::add_watch_dirs(&mut live, &id, project::watch_dirs(&full));
            }
        }
        live.next_version = live.next_version.saturating_add(1);
        let version = live.next_version;
        let cached = registry
            .cache
            .write()
            .ok()
            .and_then(|mut cache| cache.remove(&id));
        let Some(entry) = live.projects.get_mut(&id) else {
            continue;
        };
        for rel in rels {
            let new_text = read_small(&root.join(&rel));
            let old_text = entry
                .changes
                .get(&rel)
                .and_then(|change| change.text.clone())
                .or_else(|| cached.as_ref().and_then(|idx| idx.files.get(&rel).cloned()));
            let lines = match (&old_text, &new_text) {
                (Some(old), Some(new)) => project::changed_lines(old, new),
                _ => Vec::new(),
            };
            entry.changes.insert(
                rel,
                project::FileChange {
                    version,
                    at: Instant::now(),
                    lines,
                    text: new_text,
                },
            );
        }
        entry.version = version;
        entry.notify.send_replace(version);
    }
}

// needed helper: project-relative path of a change, or None for noise (hidden dirs, editor temp files)
fn relevant_rel(root: &Path, path: &Path) -> Option<PathBuf> {
    let rel = path.strip_prefix(root).ok()?;
    if rel.as_os_str().is_empty() {
        return None;
    }
    let hidden = rel
        .components()
        .any(|c| c.as_os_str().to_str().is_some_and(project::is_hidden_dir));
    if hidden {
        return None;
    }
    let name = rel.file_name()?.to_str()?;
    let temp = name.ends_with('~')
        || name.starts_with(".#")
        || name == "4913"
        || [".swp", ".swx", ".swo", ".tmp"]
            .iter()
            .any(|ext| name.ends_with(ext));
    if temp {
        return None;
    }
    Some(rel.to_path_buf())
}

// needed helper: file text for line diffs, skipping large or non-utf8 files
fn read_small(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > project::MAX_DIFF_BYTES {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Instant;

    use super::record_changes;
    use crate::index;
    use crate::project;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "fn a() {}\nfn b() {}\n").unwrap();

        let registry = project::Registry::default();
        let mut idx = index::SymbolIndex::default();
        idx.files
            .insert(PathBuf::from("src/lib.rs"), "fn a() {}\n".to_string());
        registry
            .cache
            .write()
            .unwrap()
            .insert("demo".to_string(), Arc::new(idx));
        let (notify, rx) = tokio::sync::watch::channel(1);
        {
            let mut live = registry.live.lock().unwrap();
            live.next_version = 1;
            live.projects.insert(
                "demo".to_string(),
                project::LiveProject {
                    root: root.clone(),
                    base_version: 1,
                    version: 1,
                    last_access: Instant::now(),
                    dirs: Vec::new(),
                    changes: HashMap::new(),
                    notify,
                },
            );
        }

        let mut paths = HashSet::new();
        paths.insert(root.join("src/lib.rs"));
        paths.insert(root.join("target/debug/x"));
        paths.insert(root.join("src/.lib.rs.swp"));
        record_changes(&registry, &paths);

        let (version, count, lines) = registry
            .live
            .lock()
            .map(|live| {
                let entry = &live.projects["demo"];
                (
                    entry.version,
                    entry.changes.len(),
                    entry.changes[&PathBuf::from("src/lib.rs")].lines.clone(),
                )
            })
            .unwrap();
        assert_eq!(version, 2);
        assert_eq!(count, 1);
        assert_eq!(lines, vec![2]);
        assert_eq!(*rx.borrow(), 2);
        assert!(registry.cache.read().unwrap().is_empty());
    }
}
