use std::convert::Infallible;
use std::sync::Arc;

use axum::response::sse::Event;
use futures_util::Stream;

use crate::project;

pub fn events_stream(
    registry: Arc<project::Registry>,
    id: String,
    since: u64,
    rx: tokio::sync::watch::Receiver<u64>,
) -> impl Stream<Item = Result<Event, Infallible>> {
    futures_util::stream::unfold(
        (registry, id, since, rx, true),
        |(registry, id, since, mut rx, first)| async move {
            if !first && rx.changed().await.is_err() {
                return None;
            }
            let feed = project::changes_since(&registry, &id, since)?;
            let data = serde_json::json!({
                "version": feed.version,
                "reset": feed.reset,
                "paths": feed.paths,
            });
            let event = Event::default().event("change").data(data.to_string());
            Some((Ok(event), (registry, id, feed.version, rx, false)))
        },
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Instant;

    use futures_util::StreamExt;

    use super::events_stream;
    use crate::project;

    #[tokio::test]
    async fn test_usage() {
        let registry = Arc::new(project::Registry::default());
        let (notify, rx) = tokio::sync::watch::channel(2);
        registry.live.lock().unwrap().projects.insert(
            "demo".to_string(),
            project::LiveProject {
                root: PathBuf::from("/x"),
                base_version: 2,
                version: 2,
                last_access: Instant::now(),
                dirs: Vec::new(),
                changes: HashMap::new(),
                notify,
            },
        );
        let stream = events_stream(registry.clone(), "demo".to_string(), 2, rx);
        let mut stream = Box::pin(stream);
        assert!(stream.next().await.is_some());
        registry.live.lock().unwrap().projects.remove("demo");
        assert!(stream.next().await.is_none());
    }
}
