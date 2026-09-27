use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use notify::EventKind;
use notify::event::ModifyKind;

use crate::project;

pub async fn watch_loop(
    registry: Arc<project::Registry>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<notify::Event>,
) {
    let quiet = Duration::from_millis(300);
    let max_wait = Duration::from_secs(2);
    let cooldown = Duration::from_secs(1);
    while let Some(first) = rx.recv().await {
        let mut paths: HashSet<PathBuf> = HashSet::new();
        collect(&first, &mut paths);
        let started = Instant::now();
        loop {
            let remaining = max_wait.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                break;
            }
            match tokio::time::timeout(quiet.min(remaining), rx.recv()).await {
                Ok(Some(event)) => collect(&event, &mut paths),
                Ok(None) | Err(_) => break,
            }
        }
        if paths.is_empty() {
            continue;
        }
        let target = registry.clone();
        let _ = tokio::task::spawn_blocking(move || project::record_changes(&target, &paths)).await;
        tokio::time::sleep(cooldown).await;
    }
}

// needed helper: keep only content/structure events, never reads or metadata touches
fn collect(event: &notify::Event, paths: &mut HashSet<PathBuf>) {
    let wanted = matches!(
        event.kind,
        EventKind::Create(_)
            | EventKind::Remove(_)
            | EventKind::Modify(
                ModifyKind::Data(_) | ModifyKind::Name(_) | ModifyKind::Any | ModifyKind::Other
            )
    );
    if wanted {
        paths.extend(event.paths.iter().cloned());
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Instant;

    use notify::EventKind;
    use notify::event::AccessKind;
    use notify::event::CreateKind;

    use super::watch_loop;
    use crate::project;

    #[tokio::test]
    async fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let registry = Arc::new(project::Registry::default());
        let (notify, mut changed) = tokio::sync::watch::channel(1);
        registry.live.lock().unwrap().projects.insert(
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
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let task = tokio::spawn(watch_loop(registry.clone(), rx));
        let file = dir.path().join("a.rs");
        tx.send(notify::Event::new(EventKind::Access(AccessKind::Any)).add_path(file.clone()))
            .unwrap();
        tx.send(notify::Event::new(EventKind::Create(CreateKind::File)).add_path(file))
            .unwrap();
        changed.changed().await.unwrap();
        assert_eq!(
            registry.live.lock().unwrap().projects["demo"].changes.len(),
            1
        );
        task.abort();
    }
}
