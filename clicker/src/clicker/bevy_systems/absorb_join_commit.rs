use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::clicker;
use crate::lobby;

pub fn absorb_join_commit(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    lobby: Res<clicker::ActiveLobby>,
    gate: ResMut<lobby::JoinGate>,
    own: Res<net_id::NetworkId>,
) {
    let active = lobby.into_inner();
    let own = *own.into_inner();
    let gate = gate.into_inner();
    let topic = clicker::pos_topic(active);
    let mut rest = Vec::new();
    for event in events.take_all() {
        let commit = commit_of(&event, &topic, (**active).as_str());
        let Some((peer, joiner, starts_in_ms)) = commit else {
            rest.push(event);
            continue;
        };
        if joiner != own {
            let now = std::time::Instant::now();
            let reveal_at = now
                .checked_add(std::time::Duration::from_millis(starts_in_ms))
                .unwrap_or(now);
            let fail_at = now
                .checked_add(std::time::Duration::from_secs(
                    lobby::PENDING_JOIN_FAIL_SECS,
                ))
                .unwrap_or(now);
            gate.arm_pending(lobby::PendingJoin {
                peer,
                joiner,
                reveal_at,
                fail_at,
            });
            tracing::info!(target: "clicker", room = %**active, joiner = *joiner, "join commit received");
        }
    }
    events.extend(rest);
}

// needed helper: extracts a room-matching join commit from a direct or gossiped event
fn commit_of(
    event: &p2p::Event<clicker::CursorMsg>,
    topic: &str,
    room: &str,
) -> Option<(String, net_id::NetworkId, u64)> {
    match event {
        p2p::Event::Message {
            from,
            payload:
                clicker::CursorMsg::JoinCommit {
                    room: message_room,
                    joiner,
                    starts_in_ms,
                },
        } if message_room.as_str() == room => Some((from.clone(), *joiner, *starts_in_ms)),
        p2p::Event::Gossip {
            topic: incoming,
            from,
            data,
        } if incoming == topic => {
            let clicker::CursorMsg::JoinCommit {
                room: message_room,
                joiner,
                starts_in_ms,
            } = bincode::deserialize(data).ok()?
            else {
                return None;
            };
            (message_room.as_str() == room).then(|| (from.clone(), joiner, starts_in_ms))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::absorb_join_commit;
    use crate::clicker;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(lobby::JoinGate::default());
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
                payload: clicker::CursorMsg::JoinCommit {
                    room: "alpha".to_string(),
                    joiner: net_id::NetworkId(9),
                    starts_in_ms: 5_000,
                },
            });
        app.add_systems(Update, absorb_join_commit);
        app.update();
        let gate = app.world().resource::<lobby::JoinGate>();
        assert!(
            gate.has_pending(net_id::NetworkId(9)),
            "commit arms the joiner"
        );
        assert!(
            app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .is_empty(),
            "the commit is consumed"
        );
    }

    #[test]
    fn own_commit_ignored() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer-1".to_string(),
                payload: clicker::CursorMsg::JoinCommit {
                    room: "alpha".to_string(),
                    joiner: net_id::NetworkId(1),
                    starts_in_ms: 5_000,
                },
            });
        app.add_systems(Update, absorb_join_commit);
        app.update();
        assert!(
            app.world().resource::<lobby::JoinGate>().pending.is_empty(),
            "our own commit is not re-armed from the wire"
        );
    }

    #[test]
    fn wrong_room_commit_dropped() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer-9".to_string(),
                payload: clicker::CursorMsg::JoinCommit {
                    room: "beta".to_string(),
                    joiner: net_id::NetworkId(9),
                    starts_in_ms: 5_000,
                },
            });
        app.add_systems(Update, absorb_join_commit);
        app.update();
        assert_eq!(app.world().resource::<lobby::JoinGate>().pending.len(), 0);
    }
}
