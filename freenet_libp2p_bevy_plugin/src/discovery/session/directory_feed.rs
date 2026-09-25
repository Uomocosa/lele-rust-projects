use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;

use super::super::directory::room_catalog::RoomCatalog;

#[derive(Resource, Debug, Deref)]
pub struct DirectoryFeed(pub Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<RoomCatalog>>>);

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::DirectoryFeed;

    #[test]
    fn test_usage() {
        let (_dir_tx, dir_rx) = tokio::sync::mpsc::unbounded_channel();
        let feed = DirectoryFeed(Mutex::new(Some(dir_rx)));
        assert!(feed.lock().is_ok_and(|guard| guard.is_some()));
    }
}
