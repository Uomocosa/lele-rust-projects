use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::p2p;

use crate::boxes;

pub fn apply_snapshot(
    mut snaps: MessageReader<p2p::IncomingSnapshot<boxes::Payload>>,
    mut remote: Query<
        (Entity, &boxes::Player, Option<&mut p2p::RemoteTarget>),
        Without<boxes::LocalPlayer>,
    >,
    mut commands: Commands,
) {
    for snap in snaps.read() {
        let from = snap.from;
        for (entity, player, existing) in &mut remote {
            if **player != from {
                continue;
            }
            let stale = match existing {
                Some(target) => target.tick >= snap.snapshot.tick,
                None => false,
            };
            if stale {
                continue;
            }
            commands.entity(entity).insert(p2p::RemoteTarget {
                pos: Vec2::new(snap.snapshot.payload.x, snap.snapshot.payload.y),
                tick: snap.snapshot.tick,
                sent_at_ms: snap.snapshot.sent_at_ms,
            });
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::apply_snapshot;
    use crate::boxes;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<p2p::IncomingSnapshot<boxes::Payload>>();
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<p2p::IncomingSnapshot<boxes::Payload>>>()
            .write(p2p::IncomingSnapshot {
                from: net_id::NetworkId(7),
                snapshot: p2p::Snapshot {
                    from_id: net_id::NetworkId(7),
                    tick: 3,
                    sent_at_ms: 0,
                    payload: boxes::Payload {
                        x: 11.0,
                        y: 12.0,
                        vx: 0.0,
                        vy: 0.0,
                    },
                },
            });
        let entity = app
            .world_mut()
            .spawn((boxes::Player(net_id::NetworkId(7)), Transform::default()))
            .id();
        app.add_systems(Update, apply_snapshot);

        app.update();

        let target = app.world().get::<p2p::RemoteTarget>(entity);
        assert!(target.is_some());
        assert_eq!(
            target.map(|t| (t.pos.x, t.pos.y, t.tick)),
            Some((11.0, 12.0, 3))
        );
    }
}
