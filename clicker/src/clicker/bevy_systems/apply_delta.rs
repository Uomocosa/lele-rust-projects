use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;

pub fn apply_delta(
    mut events: ResMut<p2p::Events<clicker::ClickDelta>>,
    mut targets: Query<(&clicker::Owner, &mut clicker::ClickCounter)>,
    mut global: ResMut<clicker::GlobalCounter>,
    own: Res<net_id::NetworkId>,
) {
    let own = own.into_inner();
    let mut rest = Vec::new();
    for event in events.take_all() {
        match event {
            p2p::Event::Message { from, payload } => {
                if payload.owner == *own {
                    continue;
                }
                // Credit goes to the transport sender: `payload.owner` lives in the
                // sender's local id space (CLI `--own-id`) and is advisory only.
                // Treating it as authoritative breaks delivery because it never
                // equals `from_peer(from)`. Real sender auth needs signed deltas.
                let sender = net_id::NetworkId::from_peer(&from);
                for (owner, mut counter) in &mut targets {
                    if **owner == sender {
                        counter.add(payload.delta);
                        global.add(payload.delta);
                        break;
                    }
                }
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
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
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::ClickDelta>::default());
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(net_id::NetworkId(99));
        let sender = net_id::NetworkId::from_peer("peer");
        let target = app
            .world_mut()
            .spawn((clicker::Owner(sender), clicker::ClickCounter::default()))
            .id();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::ClickDelta>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: clicker::ClickDelta {
                    owner: sender,
                    delta: 3,
                },
            });
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::ClickDelta>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: clicker::ClickDelta {
                    owner: net_id::NetworkId(99),
                    delta: 5,
                },
            });
        app.add_systems(Update, apply_delta);
        app.update();
        assert_eq!(
            **app.world().get::<clicker::ClickCounter>(target).unwrap(),
            3
        );
        assert_eq!(**app.world().resource::<clicker::GlobalCounter>(), 3);
        assert!(
            app.world()
                .resource::<p2p::Events<clicker::ClickDelta>>()
                .is_empty()
        );
    }

    #[test]
    fn test_sender_credited_owner_advisory() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::ClickDelta>::default());
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(net_id::NetworkId(99));
        let sender = net_id::NetworkId::from_peer("peer");
        let target = app
            .world_mut()
            .spawn((clicker::Owner(sender), clicker::ClickCounter::default()))
            .id();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::ClickDelta>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: clicker::ClickDelta {
                    owner: net_id::NetworkId(7),
                    delta: 10,
                },
            });
        app.add_systems(Update, apply_delta);
        app.update();
        assert_eq!(
            **app.world().get::<clicker::ClickCounter>(target).unwrap(),
            10
        );
        assert_eq!(**app.world().resource::<clicker::GlobalCounter>(), 10);
    }
}
