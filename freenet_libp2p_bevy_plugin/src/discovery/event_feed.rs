use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;

use super::Event;

#[derive(Resource, Debug, Deref)]
pub struct EventFeed(pub Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<Event>>>);

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::EventFeed;

    #[test]
    fn test_usage() {
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let feed = EventFeed(Mutex::new(Some(rx)));
        assert!(feed.lock().is_ok_and(|guard| guard.is_some()));
    }
}
