use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;

pub fn apply_delta(
    mut snaps: MessageReader<p2p::IncomingSnapshot<clicker::ClickDelta>>,
    mut counters: Query<(&clicker::Owner, &mut clicker::ClickCounter)>,
) {
    for snap in snaps.read() {
        let from = snap.from;
        for (owner, mut counter) in &mut counters {
            if **owner == from {
                **counter += *snap.snapshot.payload;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::apply_delta;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<p2p::IncomingSnapshot<clicker::ClickDelta>>();
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(7)),
            clicker::ClickCounter(3),
        ));
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<p2p::IncomingSnapshot<clicker::ClickDelta>>>()
            .write(p2p::IncomingSnapshot {
                from: net_id::NetworkId(7),
                snapshot: p2p::Snapshot {
                    from_id: net_id::NetworkId(7),
                    tick: 0,
                    sent_at_ms: 0,
                    payload: clicker::ClickDelta(2),
                },
            });
        app.add_systems(Update, apply_delta);
        app.update();

        let mut query = app.world_mut().query::<&clicker::ClickCounter>();
        let counter = query.single(app.world()).unwrap();
        assert_eq!(**counter, 5);
    }
}
