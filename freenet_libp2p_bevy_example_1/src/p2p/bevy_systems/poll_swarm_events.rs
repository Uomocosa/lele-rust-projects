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
    while let Ok(event) = events.try_recv() {
        match event {
            p2p::Event::IncomingSnapshot { snapshot, .. } => {
                let player_id = boxes::PlayerId(snapshot.player_id);
                for (entity, player, target) in &mut remote_boxes {
                    if **player != player_id {
                        continue;
                    }
                    let stale = match target {
                        Some(target) => target.tick >= snapshot.tick,
                        None => false,
                    };
                    if stale {
                        continue;
                    }
                    commands.entity(entity).insert(p2p::RemoteTarget {
                        pos: Vec2::new(snapshot.x, snapshot.y),
                        tick: snapshot.tick,
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
                for (entity, player, _) in &mut remote_boxes {
                    if **player == player_id {
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
            .spawn((boxes::Player(boxes::PlayerId(7)), Transform::default()))
            .id();
        app.add_systems(Update, poll_swarm_events);

        event_tx
            .send(p2p::Event::IncomingSnapshot {
                from: libp2p::PeerId::random(),
                snapshot: p2p::Snapshot {
                    player_id: 7,
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
        let peer_id = libp2p::PeerId::random();
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
}
