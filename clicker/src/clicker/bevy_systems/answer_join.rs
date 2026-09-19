use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::p2p;
use freenet_libp2p_bevy_plugin::roster;

use crate::clicker;

pub fn answer_join(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
    lobby: Res<clicker::ActiveLobby>,
    members: Res<roster::Roster>,
    own: Res<net_id::NetworkId>,
    targets: Query<(&clicker::Owner, &clicker::ClickCounter)>,
) {
    let lobby = lobby.into_inner();
    let members = members.into_inner();
    let own = *own.into_inner();
    let commands = commands.into_inner();
    let mut rest = Vec::new();
    for event in events.take_all() {
        match event {
            p2p::Event::Message {
                from,
                payload: clicker::CursorMsg::WantJoin { room },
            } if room.as_str() == lobby.as_str() => {
                let peers = members
                    .get(&room)
                    .map(|entries| {
                        entries
                            .values()
                            .filter(|peer| peer.as_str() != from.as_str())
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();
                let mut score = 0;
                for (owner, counter) in &targets {
                    if ***owner == *own {
                        score = **counter;
                        break;
                    }
                }
                tracing::info!(target: "clicker", room = %room, from = %from, "join: welcome sent");
                commands.push(p2p::Command::Send {
                    peer_id: from,
                    payload: clicker::CursorMsg::Welcome {
                        room,
                        peers,
                        own_score: (own, score),
                        joining: Vec::new(),
                    },
                });
            }
            p2p::Event::Message {
                payload: clicker::CursorMsg::WantJoin { .. },
                ..
            } => {
                tracing::debug!(target: "clicker", "join: want for another room dropped");
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::answer_join;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(roster::Roster::default());
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter(4),
        ));
        app
    }

    #[test]
    fn test_usage() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer-9".to_string(),
                payload: clicker::CursorMsg::WantJoin {
                    room: "alpha".to_string(),
                },
            });
        app.add_systems(Update, answer_join);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
        assert!(
            commands.iter().any(|command| matches!(
                command,
                p2p::Command::Send { peer_id, payload }
                    if peer_id == "peer-9"
                        && matches!(
                            payload,
                            clicker::CursorMsg::Welcome {
                                room,
                                own_score,
                                joining,
                                ..
                            } if room == "alpha"
                                && *own_score == (net_id::NetworkId(1), 4)
                                && joining.is_empty()
                        )
            )),
            "join request draws a welcome carrying our own authoritative score"
        );
        assert!(
            app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .is_empty(),
            "the join request is consumed"
        );
    }

    #[test]
    fn welcome_never_advertises_the_requester() {
        let mut app = test_app();
        {
            let mut roster = app.world_mut().resource_mut::<roster::Roster>();
            roster.add_entry(
                "alpha".to_string(),
                *blake3::hash(b"peer-9").as_bytes(),
                "peer-9".to_string(),
            );
            roster.add_entry(
                "alpha".to_string(),
                *blake3::hash(b"peer-8").as_bytes(),
                "peer-8".to_string(),
            );
        }
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer-9".to_string(),
                payload: clicker::CursorMsg::WantJoin {
                    room: "alpha".to_string(),
                },
            });
        app.add_systems(Update, answer_join);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        let peers = commands
            .iter()
            .find_map(|command| match command {
                p2p::Command::Send {
                    payload: clicker::CursorMsg::Welcome { peers, .. },
                    ..
                } => Some(peers.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(
            !peers.contains(&"peer-9".to_string()),
            "no self echo: {peers:?}"
        );
        assert!(peers.contains(&"peer-8".to_string()));
    }

    #[test]
    fn wrong_room_want_dropped() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer-9".to_string(),
                payload: clicker::CursorMsg::WantJoin {
                    room: "beta".to_string(),
                },
            });
        app.add_systems(Update, answer_join);
        app.update();
        assert!(
            app.world()
                .resource::<p2p::Commands<clicker::CursorMsg>>()
                .is_empty(),
            "join request for another room draws no welcome"
        );
    }
}
