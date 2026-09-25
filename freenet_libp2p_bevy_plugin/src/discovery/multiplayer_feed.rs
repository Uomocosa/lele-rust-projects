use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;

use super::Multiplayer;

#[derive(Resource, Debug, Deref)]
pub struct MultiplayerFeed(pub Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<Multiplayer>>>);

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::MultiplayerFeed;

    #[test]
    fn test_usage() {
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let feed = MultiplayerFeed(Mutex::new(Some(rx)));
        assert!(feed.lock().is_ok_and(|guard| guard.is_some()));
    }
}
