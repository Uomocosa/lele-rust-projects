#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::super::constants;
use super::super::super::directory::directory_state::DirectoryState;
use super::super::super::directory::room_entry::RoomEntry;
use super::super::super::directory::room_list::RoomList;
use super::super::super::session::directory_feed::DirectoryFeed;
use super::super::super::session::directory_live::DirectoryLive;

pub fn poll_directory(
    feed: Res<DirectoryFeed>,
    mut rooms: ResMut<RoomList>,
    mut live: ResMut<DirectoryLive>,
) {
    let Ok(mut guard) = feed.lock() else {
        return;
    };
    let Some(rx) = guard.as_mut() else {
        return;
    };
    let mut latest: Option<DirectoryState> = None;
    while let Ok(state) = rx.try_recv() {
        latest = Some(state);
    }
    let Some(state) = latest else {
        return;
    };
    if !**live {
        **live = true;
        tracing::info!(target: "room_lobby", "menu live: directory connected");
    }
    let mut entries: Vec<RoomEntry> = state
        .iter()
        .map(|(name, entry)| RoomEntry {
            name: name.clone(),
            updated_at: entry.updated_at,
        })
        .collect();
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.updated_at));
    entries.truncate(constants::MENU_MAX_ROOMS);
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
    rooms.entries = entries;
    rooms.revision = rooms.revision.wrapping_add(1);
    tracing::info!(target: "room_lobby", rooms = ?names, "directory listed rooms");
}

#[cfg(test)]
mod tests {
    use super::poll_directory;
    use crate::discovery;
    use bevy::prelude::*;
    use std::sync::Mutex;

    #[test]
    fn test_usage() {
        let (tx, rx) =
            tokio::sync::mpsc::unbounded_channel::<discovery::directory::DirectoryState>();
        let mut state = discovery::directory::DirectoryState::new();
        state.insert(
            "room-old".to_string(),
            discovery::directory::Entry {
                params: Vec::new(),
                peer_id: "peer".to_string(),
                addrs: Vec::new(),
                updated_at: 5,
            },
        );
        state.insert(
            "room-new".to_string(),
            discovery::directory::Entry {
                params: Vec::new(),
                peer_id: "peer".to_string(),
                addrs: Vec::new(),
                updated_at: 9,
            },
        );
        tx.send(state).expect("send");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(discovery::session::DirectoryFeed(Mutex::new(Some(rx))));
        app.insert_resource(discovery::directory::RoomList::default());
        app.insert_resource(discovery::session::DirectoryLive::default());
        app.add_systems(Update, poll_directory);
        app.update();
        let rooms = app.world().resource::<discovery::directory::RoomList>();
        assert_eq!(rooms.entries.len(), 2);
        assert_eq!(rooms.entries[0].name, "room-new");
        assert_eq!(rooms.revision, 1);
        assert!(**app.world().resource::<discovery::session::DirectoryLive>());
    }

    #[test]
    fn silent_feed_keeps_menu_dark() {
        let (_tx, rx) =
            tokio::sync::mpsc::unbounded_channel::<discovery::directory::DirectoryState>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(discovery::session::DirectoryFeed(Mutex::new(Some(rx))));
        app.insert_resource(discovery::directory::RoomList::default());
        app.insert_resource(discovery::session::DirectoryLive::default());
        app.add_systems(Update, poll_directory);
        app.update();
        assert!(!**app.world().resource::<discovery::session::DirectoryLive>());
    }
}
