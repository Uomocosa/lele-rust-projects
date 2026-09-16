use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::p2p;
use freenet_libp2p_bevy_plugin::roster;

use crate::clicker;
use crate::lobby;

pub fn absorb_welcome(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    lobby: Res<clicker::ActiveLobby>,
    gate: ResMut<lobby::JoinGate>,
    clock: ResMut<lobby::JoinClock>,
    members: ResMut<roster::Roster>,
    own: Res<net_id::NetworkId>,
) {
    let lobby = lobby.into_inner();
    let gate = gate.into_inner();
    let clock = clock.into_inner();
    let members = members.into_inner();
    let own = *own.into_inner();
    let mut rest = Vec::new();
    for event in events.take_all() {
        match event {
            p2p::Event::Message {
                from,
                payload:
                    clicker::CursorMsg::Welcome {
                        room,
                        peers,
                        own_score,
                        joining,
                    },
            } if room.as_str() == lobby.as_str() => {
                tracing::info!(target: "clicker", room = %room, from = %from, peers = peers.len(), "join: welcome absorbed");
                let known: Vec<String> = members
                    .get(&room)
                    .map(|entries| entries.values().cloned().collect())
                    .unwrap_or_default();
                let mut union = gate.expected.clone().unwrap_or_default();
                let before = union.len();
                for peer in core::iter::once(&from)
                    .chain(peers.iter())
                    .chain(joining.iter())
                {
                    union.insert(peer.clone());
                    if known.iter().any(|entry| entry == peer) {
                        continue;
                    }
                    let key = *blake3::hash(peer.as_bytes()).as_bytes();
                    members.add_entry(room.clone(), key, peer.clone());
                }
                if union.len() > before {
                    clock.last_new_peer = Some(std::time::Instant::now());
                }
                gate.expected = Some(union);
                rest.push(p2p::Event::Message {
                    from,
                    payload: clicker::CursorMsg::SyncAck {
                        target: own,
                        entries: vec![own_score],
                    },
                });
            }
            p2p::Event::Message {
                payload: clicker::CursorMsg::Welcome { .. },
                ..
            } => {
                tracing::debug!(target: "clicker", "join: welcome for another room dropped");
            }
            p2p::Event::Message {
                payload: clicker::CursorMsg::WantJoin { .. },
                ..
            } => {
                tracing::debug!(target: "clicker", "join: stray want dropped at welcome");
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::absorb_welcome;
    use crate::clicker;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(lobby::JoinClock::default());
        app.insert_resource(roster::Roster::default());
        app.insert_resource(net_id::NetworkId(1));
        app
    }

    #[test]
    fn test_usage() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer-9".to_string(),
                payload: clicker::CursorMsg::Welcome {
                    room: "alpha".to_string(),
                    peers: vec!["peer-8".to_string()],
                    own_score: (net_id::NetworkId(9), 5),
                    joining: Vec::new(),
                },
            });
        app.add_systems(Update, absorb_welcome);
        app.update();
        let gate = app.world().resource::<lobby::JoinGate>();
        assert!(
            gate.expected
                .as_ref()
                .is_some_and(|set| { set.contains("peer-9") && set.contains("peer-8") }),
            "sender plus advertised peers union into expected"
        );
        let members = app
            .world()
            .resource::<roster::Roster>()
            .get(&"alpha".to_string())
            .map(|entries| entries.values().cloned().collect::<Vec<String>>())
            .unwrap_or_default();
        assert!(
            members.contains(&"peer-9".to_string()) && members.contains(&"peer-8".to_string()),
            "welcome peers enter the roster, got {members:?}"
        );
        assert!(
            app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .iter()
                .any(|event| matches!(
                    event,
                    p2p::Event::Message {
                        payload: clicker::CursorMsg::SyncAck { .. },
                        ..
                    }
                )),
            "the welcome score re-enters as a sync ack for the existing merge path"
        );
    }

    #[test]
    fn wrong_room_welcome_dropped() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer-9".to_string(),
                payload: clicker::CursorMsg::Welcome {
                    room: "beta".to_string(),
                    peers: vec!["peer-8".to_string()],
                    own_score: (net_id::NetworkId(9), 5),
                    joining: Vec::new(),
                },
            });
        app.add_systems(Update, absorb_welcome);
        app.update();
        assert!(
            app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .is_empty(),
            "welcome for another room is consumed without effect"
        );
        assert!(
            app.world().resource::<lobby::JoinGate>().expected.is_none(),
            "expected stays unknown on wrong-room welcome"
        );
    }
}
