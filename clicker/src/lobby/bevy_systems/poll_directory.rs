use bevy::prelude::*;

use crate::discovery;
use crate::lobby;

pub fn poll_directory(
    feed: Res<lobby::DirectoryFeed>,
    rooms: ResMut<lobby::RoomList>,
    live: ResMut<lobby::DirectoryLive>,
) {
    let feed = feed.into_inner();
    let Ok(mut guard) = feed.lock() else {
        return;
    };
    let Some(rx) = guard.as_mut() else {
        return;
    };
    let mut latest: Option<discovery::DirectoryState> = None;
    while let Ok(state) = rx.try_recv() {
        latest = Some(state);
    }
    let Some(state) = latest else {
        return;
    };
    let live = live.into_inner();
    if !**live {
        **live = true;
        tracing::info!(target: "clicker", "menu live: directory connected");
    }
    let mut entries: Vec<lobby::RoomEntry> = state
        .iter()
        .map(|(name, entry)| lobby::RoomEntry {
            name: name.clone(),
            updated_at: entry.updated_at,
        })
        .collect();
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.updated_at));
    entries.truncate(lobby::MENU_MAX_ROOMS);
    let same = rooms.entries.len() == entries.len()
        && rooms
            .entries
            .iter()
            .zip(entries.iter())
            .all(|(current, next)| {
                current.name == next.name && current.updated_at == next.updated_at
            });
    if same {
        return;
    }
    let names: Vec<String> = entries.iter().map(|entry| entry.name.clone()).collect();
    let rooms = rooms.into_inner();
    rooms.entries = entries;
    rooms.revision = rooms.revision.wrapping_add(1);
    tracing::info!(target: "clicker", rooms = ?names, "directory listed rooms");
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
        app.insert_resource(lobby::DirectoryLive::default());
        app.add_systems(Update, poll_directory);
        app.update();
        let rooms = app.world().resource::<lobby::RoomList>();
        assert_eq!(rooms.entries.len(), 2);
        assert_eq!(rooms.entries[0].name, "room-new");
        assert_eq!(rooms.entries[1].name, "room-old");
        assert_eq!(rooms.revision, 1);
    }

    #[test]
    fn caps_menu_to_recent_rooms() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<discovery::DirectoryState>();
        let mut state = discovery::DirectoryState::new();
        let cap = u64::try_from(lobby::MENU_MAX_ROOMS).unwrap_or_default();
        for index in 0..cap.saturating_add(5) {
            state.insert(
                format!("room-{index}"),
                discovery::DirectoryEntry {
                    params: Vec::new(),
                    peer_id: "peer".to_string(),
                    addrs: Vec::new(),
                    updated_at: index,
                },
            );
        }
        tx.send(state).expect("send");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(lobby::DirectoryFeed(Mutex::new(Some(rx))));
        app.insert_resource(lobby::RoomList::default());
        app.insert_resource(lobby::DirectoryLive::default());
        app.add_systems(Update, poll_directory);
        app.update();
        let rooms = app.world().resource::<lobby::RoomList>();
        assert_eq!(rooms.entries.len(), lobby::MENU_MAX_ROOMS);
        assert_eq!(
            rooms.entries[0].name,
            format!("room-{}", cap.saturating_add(4))
        );
    }

    #[test]
    fn first_feed_marks_menu_live() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<discovery::DirectoryState>();
        tx.send(discovery::DirectoryState::new()).expect("send");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(lobby::DirectoryFeed(Mutex::new(Some(rx))));
        app.insert_resource(lobby::RoomList::default());
        app.insert_resource(lobby::DirectoryLive::default());
        app.add_systems(Update, poll_directory);
        assert!(!**app.world().resource::<lobby::DirectoryLive>());
        app.update();
        assert!(
            **app.world().resource::<lobby::DirectoryLive>(),
            "first directory feed enables the menu"
        );
    }

    #[test]
    fn silent_feed_keeps_menu_dark() {
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel::<discovery::DirectoryState>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(lobby::DirectoryFeed(Mutex::new(Some(rx))));
        app.insert_resource(lobby::RoomList::default());
        app.insert_resource(lobby::DirectoryLive::default());
        app.add_systems(Update, poll_directory);
        app.update();
        assert!(
            !**app.world().resource::<lobby::DirectoryLive>(),
            "no feed means no live menu"
        );
    }
}
