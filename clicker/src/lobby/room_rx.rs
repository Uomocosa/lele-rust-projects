use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Debug, Deref)]
pub struct RoomRx(pub Mutex<Option<tokio::sync::watch::Receiver<Option<String>>>>);

#[cfg(test)]
mod tests {
    use super::RoomRx;
    use bevy::prelude::*;
    use std::sync::Mutex;

    #[test]
    fn test_usage() {
        let (tx, rx) = tokio::sync::watch::channel(None::<String>);
        let feed = RoomRx(Mutex::new(Some(rx)));
        assert!(feed.lock().is_ok_and(|guard| guard.is_some()));
        tx.send_replace(Some("room-a".to_string()));
        let room = feed
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().and_then(|rx| rx.borrow().clone()));
        assert_eq!(room, Some("room-a".to_string()));
    }
}
