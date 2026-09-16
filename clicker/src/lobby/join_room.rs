use freenet_libp2p_bevy_plugin::p2p;
use freenet_libp2p_bevy_plugin::roster;

use crate::clicker;
use crate::constants;
use crate::lobby;

pub fn join_room(
    room: &str,
    rooms: &mut lobby::JoinRooms<'_>,
    commands: &mut p2p::Commands<clicker::CursorMsg>,
) {
    *rooms.active = clicker::ActiveLobby(room.to_string());
    **rooms.selected = Some(room.to_string());
    *rooms.roster_lobby = roster::Lobby(room.to_string());
    **rooms.pending = Some(room.to_string());
    rooms.gate.expected = None;
    rooms.gate.synced.clear();
    rooms.clock.clicked_at = Some(std::time::Instant::now());
    rooms.clock.last_new_peer = None;
    let active = clicker::ActiveLobby(room.to_string());
    commands.push(p2p::Command::FetchHistory {
        lobby: room.to_string(),
        chunk: constants::SNAPSHOT_CHUNK,
    });
    commands.push(p2p::Command::FetchRoster {
        lobby: room.to_string(),
    });
    commands.push(p2p::Command::Subscribe {
        topic: clicker::pos_topic(&active),
    });
    commands.push(p2p::Command::Subscribe {
        topic: clicker::click_topic(&active),
    });
    commands.push(p2p::Command::Subscribe {
        topic: clicker::gossip_roster_topic(&active),
    });
}

#[cfg(test)]
mod tests {
    use super::join_room;
    use crate::clicker;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::p2p;

    #[test]
    fn test_usage() {
        use std::collections::BTreeSet;

        use freenet_libp2p_bevy_plugin::roster;

        let mut active = clicker::ActiveLobby::default();
        let mut selected = lobby::SelectedRoom::default();
        let mut roster_lobby = roster::Lobby::default();
        let mut pending = lobby::JoinPending::default();
        let mut gate = lobby::JoinGate {
            expected: Some(BTreeSet::from(["old-room-peer".to_string()])),
            synced: vec![lobby::SyncedPeer("old-room-peer".to_string())],
        };
        let mut clock = lobby::JoinClock::default();
        let mut commands = p2p::Commands::<clicker::CursorMsg>::default();
        join_room(
            "room-a",
            &mut lobby::JoinRooms {
                active: &mut active,
                selected: &mut selected,
                roster_lobby: &mut roster_lobby,
                pending: &mut pending,
                gate: &mut gate,
                clock: &mut clock,
            },
            &mut commands,
        );
        assert_eq!(active, clicker::ActiveLobby("room-a".to_string()));
        assert_eq!(*selected, Some("room-a".to_string()));
        assert_eq!(roster_lobby.as_str(), "room-a");
        assert_eq!(*pending, Some("room-a".to_string()));
        assert!(gate.expected.is_none(), "stale expected set resets on join");
        assert!(gate.synced.is_empty(), "stale sync record resets on join");
        assert!(clock.clicked_at.is_some(), "click starts the alone-cap");
        assert!(
            clock.last_new_peer.is_none(),
            "no peers discovered yet at click"
        );
        assert_eq!(commands.len(), 5);
        let topics: Vec<String> = commands
            .iter()
            .filter_map(|command| match command {
                p2p::Command::Subscribe { topic } => Some(topic.clone()),
                _ => None,
            })
            .collect();
        assert!(topics.contains(&"clicker/room-a/pos".to_string()));
        assert!(topics.contains(&"clicker/room-a/click".to_string()));
        assert!(topics.contains(&"clicker/room-a/roster".to_string()));
        assert!(commands.iter().any(|command| matches!(
            command,
            p2p::Command::FetchHistory { lobby, chunk: 0 }
            if lobby == "room-a"
        )));
        assert!(commands.iter().any(|command| matches!(
            command,
            p2p::Command::FetchRoster { lobby }
            if lobby == "room-a"
        )));
    }
}
