use freenet_libp2p_bevy_plugin::p2p;
use freenet_libp2p_bevy_plugin::roster;

use crate::clicker;
use crate::constants;
use crate::lobby;

pub fn join_room(
    room: &str,
    lobby: &mut clicker::ActiveLobby,
    selected: &mut lobby::SelectedRoom,
    roster_lobby: &mut roster::Lobby,
    commands: &mut p2p::Commands<clicker::CursorMsg>,
) {
    *lobby = clicker::ActiveLobby(room.to_string());
    **selected = Some(room.to_string());
    *roster_lobby = roster::Lobby(room.to_string());
    let active = clicker::ActiveLobby(room.to_string());
    commands.push(p2p::Command::FetchHistory {
        lobby: room.to_string(),
        chunk: constants::SNAPSHOT_CHUNK,
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
        use freenet_libp2p_bevy_plugin::roster;

        let mut active = clicker::ActiveLobby::default();
        let mut selected = lobby::SelectedRoom::default();
        let mut roster_lobby = roster::Lobby::default();
        let mut commands = p2p::Commands::<clicker::CursorMsg>::default();
        join_room(
            "room-a",
            &mut active,
            &mut selected,
            &mut roster_lobby,
            &mut commands,
        );
        assert_eq!(active, clicker::ActiveLobby("room-a".to_string()));
        assert_eq!(*selected, Some("room-a".to_string()));
        assert_eq!(roster_lobby.as_str(), "room-a");
        assert_eq!(commands.len(), 4);
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
    }
}
