use std::collections::HashSet;

use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;

pub fn send_want_join(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
    lobby: Res<clicker::ActiveLobby>,
    mut live: Local<HashSet<String, std::hash::RandomState>>,
) {
    let lobby = lobby.into_inner();
    let commands = commands.into_inner();
    let room = (**lobby).clone();
    let mut rest = Vec::new();
    for event in events.take_all() {
        match event {
            p2p::Event::PeerConnected(peer) => {
                if !room.is_empty() && live.insert(peer.clone()) {
                    tracing::debug!(target: "clicker", peer = %peer, room = %room, "join: want sent");
                    commands.push(p2p::Command::Send {
                        peer_id: peer.clone(),
                        payload: clicker::CursorMsg::WantJoin { room: room.clone() },
                    });
                }
                rest.push(p2p::Event::PeerConnected(peer));
            }
            p2p::Event::PeerDisconnected(peer) => {
                live.remove(peer.as_str());
                rest.push(p2p::Event::PeerDisconnected(peer));
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::send_want_join;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::p2p;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::PeerConnected("peer".to_string()));
        app.add_systems(Update, send_want_join);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
        assert!(
            commands.iter().any(|command| matches!(
                command,
                p2p::Command::Send {
                    payload: clicker::CursorMsg::WantJoin { .. },
                    ..
                }
            )),
            "one live peer draws exactly one join request"
        );
        assert_eq!(
            app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .len(),
            1,
            "the connect event passes downstream to the roster"
        );
    }

    #[test]
    fn duplicate_connect_requests_once() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.add_systems(Update, send_want_join);
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<p2p::Events<clicker::CursorMsg>>()
                .push(p2p::Event::PeerConnected("peer".to_string()));
            app.update();
        }
        assert_eq!(
            app.world()
                .resource::<p2p::Commands<clicker::CursorMsg>>()
                .len(),
            1,
            "one live peer draws exactly one join request"
        );
    }

    #[test]
    fn empty_lobby_requests_nothing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby(String::new()));
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::PeerConnected("peer".to_string()));
        app.add_systems(Update, send_want_join);
        app.update();
        assert!(
            app.world()
                .resource::<p2p::Commands<clicker::CursorMsg>>()
                .is_empty(),
            "menu state with no room sends no join request"
        );
    }
}
