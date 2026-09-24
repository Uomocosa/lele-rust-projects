use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Debug, Deref)]
pub struct RoomRx(pub Mutex<Option<tokio::sync::watch::Receiver<Option<String>>>>);

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::RoomRx;

    #[test]
    fn test_usage() {
        let (_room_tx, room_rx) = tokio::sync::watch::channel(None::<String>);
        let feed = RoomRx(Mutex::new(Some(room_rx)));
        assert!(feed.lock().is_ok_and(|guard| guard.is_some()));
    }
}
