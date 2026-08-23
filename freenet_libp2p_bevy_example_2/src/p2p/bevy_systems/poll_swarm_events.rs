use std::collections::HashSet;

use bevy::prelude::*;

use crate::boxes;
use crate::p2p;

pub fn poll_swarm_events(
    mut events: ResMut<p2p::P2pEvents>,
    mut commands: Commands,
    mut remote_boxes: Query<
        (Entity, &boxes::Player, Option<&mut p2p::RemoteTarget>),
        Without<boxes::LocalPlayer>,
    >,
    mut peer_status: ResMut<p2p::PeerStatus>,
) {
    // `Commands` are deferred, so an entity despawned earlier in this drain is still visible in
    // `remote_boxes`. Queuing another command on it would panic when the buffers are applied — a
    // buffered snapshot arriving after a disconnect, or two disconnects for the same peer, both hit
    // that. Track the entities we already despawned and skip them.
    let mut despawned = HashSet::new();

    while let Ok(event) = events.try_recv() {
        match event {
            p2p::Event::IncomingSnapshot { snapshot, .. } => {
                let player_id = snapshot.player_id;
                for (entity, player, target) in &mut remote_boxes {
                    if **player != player_id || despawned.contains(&entity) {
                        continue;
                    }
                    let stale = match target {
                        Some(target) => target.tick >= snapshot.tick,
                        None => false,
                    };
                    if stale {
                        continue;
                    }
                    tracing::debug!(
                        target: "p2p",
                        player_id = %hex::encode(snapshot.player_id),
                        tick = snapshot.tick,
                        x = snapshot.x,
                        y = snapshot.y,
                        "applied remote snapshot"
                    );
                    commands.entity(entity).insert(p2p::RemoteTarget {
                        pos: Vec2::new(snapshot.x, snapshot.y),
                        tick: snapshot.tick,
                        sent_at_ms: snapshot.sent_at_ms,
                    });
                    break;
                }
            }
            p2p::Event::PeerConnected(peer_id) => {
                peer_status.insert(peer_id.to_base58());
            }
            p2p::Event::PeerDisconnected(peer_id) => {
                peer_status.remove(&peer_id.to_base58());
                let player_id = p2p::peer_id_to_player_id(&peer_id);
                tracing::debug!(
                    target: "p2p",
                    player_id = %hex::encode(player_id),
                    "peer disconnected, despawning box"
                );
                for (entity, player, _) in &mut remote_boxes {
                    if **player == player_id && despawned.insert(entity) {
                        commands.entity(entity).despawn();
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::poll_swarm_events;
    use crate::boxes;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event>();
        let mut app = App::new();
        app.insert_resource(p2p::P2pEvents(event_rx));
        app.insert_resource(p2p::PeerStatus::default());
        let entity = app
            .world_mut()
            .spawn((boxes::Player([7; 32]), Transform::default()))
            .id();
        app.add_systems(Update, poll_swarm_events);

        event_tx
            .send(p2p::Event::IncomingSnapshot {
                from: libp2p::identity::Keypair::generate_ed25519()
                    .public()
                    .to_peer_id(),
                snapshot: p2p::Snapshot {
                    player_id: [7; 32],
                    x: 11.0,
                    y: 12.0,
                    vx: 0.0,
                    vy: 0.0,
                    tick: 3,
                    sent_at_ms: 0,
                },
            })
            .ok();
        app.update();

        let target = app.world().get::<p2p::RemoteTarget>(entity);
        assert!(target.is_some());
        assert_eq!(
            target.map(|t| (t.pos.x, t.pos.y, t.tick)),
            Some((11.0, 12.0, 3))
        );
    }

    #[test]
    fn disconnected_peer_box_is_despawned() {
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event>();
        let peer_id = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let player_id = p2p::peer_id_to_player_id(&peer_id);
        let mut app = App::new();
        app.insert_resource(p2p::P2pEvents(event_rx));
        app.insert_resource(p2p::PeerStatus::default());
        let entity = app
            .world_mut()
            .spawn((boxes::Player(player_id), Transform::default()))
            .id();
        app.add_systems(Update, poll_swarm_events);

        event_tx.send(p2p::Event::PeerDisconnected(peer_id)).ok();
        app.update();

        assert!(app.world().get_entity(entity).is_err());
    }

    /// A snapshot buffered behind a `PeerDisconnected` for the same peer used to queue an `insert`
    /// on the entity the disconnect had already queued a `despawn` for — both land in the same
    /// command buffer and the app panicked when it was applied.
    #[test]
    fn snapshot_after_disconnect_in_one_drain_does_not_panic() {
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event>();
        let peer_id = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let player_id = p2p::peer_id_to_player_id(&peer_id);
        let mut app = App::new();
        app.insert_resource(p2p::P2pEvents(event_rx));
        app.insert_resource(p2p::PeerStatus::default());
        let entity = app
            .world_mut()
            .spawn((boxes::Player(player_id), Transform::default()))
            .id();
        app.add_systems(Update, poll_swarm_events);

        event_tx.send(p2p::Event::PeerDisconnected(peer_id)).ok();
        event_tx
            .send(p2p::Event::IncomingSnapshot {
                from: peer_id,
                snapshot: p2p::Snapshot {
                    player_id,
                    x: 11.0,
                    y: 12.0,
                    vx: 0.0,
                    vy: 0.0,
                    tick: 3,
                    sent_at_ms: 0,
                },
            })
            .ok();
        event_tx.send(p2p::Event::PeerDisconnected(peer_id)).ok();
        app.update();

        assert!(app.world().get_entity(entity).is_err());
    }
}
