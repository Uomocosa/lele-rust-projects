use bevy::prelude::*;

use crate::lobby;

pub fn poll_directory(feed: Res<lobby::DirectoryFeed>, rooms: ResMut<lobby::RoomList>) {
    let feed = feed.into_inner();
    let rooms = rooms.into_inner();
    let Ok(mut guard) = feed.lock() else {
        return;
    };
    let Some(rx) = guard.as_mut() else {
        return;
    };
    while let Ok(state) = rx.try_recv() {
        let mut entries: Vec<lobby::RoomEntry> = state
            .iter()
            .map(|(name, entry)| lobby::RoomEntry {
                name: name.clone(),
                updated_at: entry.updated_at,
            })
            .collect();
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.updated_at));
        rooms.entries = entries;
        rooms.revision = rooms.revision.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::poll_directory;
    use crate::discovery;
    use crate::lobby;
    use bevy::prelude::*;
    use std::sync::Mutex;

    #[test]
    fn test_usage() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<discovery::DirectoryState>();
        let mut state = discovery::DirectoryState::new();
        state.insert(
            "room-old".to_string(),
            discovery::DirectoryEntry {
                params: Vec::new(),
                peer_id: "peer".to_string(),
                addrs: Vec::new(),
                updated_at: 5,
            },
        );
        state.insert(
            "room-new".to_string(),
            discovery::DirectoryEntry {
                params: Vec::new(),
                peer_id: "peer".to_string(),
                addrs: Vec::new(),
                updated_at: 9,
            },
        );
        tx.send(state).expect("send");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(lobby::DirectoryFeed(Mutex::new(Some(rx))));
        app.insert_resource(lobby::RoomList::default());
        app.add_systems(Update, poll_directory);
        app.update();
        let rooms = app.world().resource::<lobby::RoomList>();
        assert_eq!(rooms.entries.len(), 2);
        assert_eq!(rooms.entries[0].name, "room-new");
        assert_eq!(rooms.entries[1].name, "room-old");
        assert_eq!(rooms.revision, 1);
    }
}
