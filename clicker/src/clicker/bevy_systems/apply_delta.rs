use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;

pub fn apply_delta(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    mut targets: Query<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    mut pending: ResMut<clicker::PendingClicks>,
    own: Res<net_id::NetworkId>,
) {
    let own = own.into_inner();
    let mut rest = Vec::new();
    for event in events.take_all() {
        match event {
            p2p::Event::Message { from, payload } => match payload {
                clicker::CursorMsg::Click { owner, delta } => {
                    if owner == *own {
                        continue;
                    }
                    if credit_logical(&mut targets, owner, delta) {
                        continue;
                    }
                    let sender = net_id::NetworkId::from_peer(&from);
                    if credit_sender(&mut targets, sender, delta) {
                        pending.items.push(clicker::PendingClick {
                            sender,
                            owner,
                            delta: 0,
                            absolute: false,
                        });
                        continue;
                    }
                    pending.items.push(clicker::PendingClick {
                        sender,
                        owner,
                        delta,
                        absolute: false,
                    });
                }
                sync
                @ (clicker::CursorMsg::SyncReq { .. } | clicker::CursorMsg::SyncAck { .. }) => {
                    rest.push(p2p::Event::Message {
                        from,
                        payload: sync,
                    });
                }
                moved @ clicker::CursorMsg::Move { .. } => {
                    rest.push(p2p::Event::Message {
                        from,
                        payload: moved,
                    });
                }
            },
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

// needed helper: credits slots keyed by logical player id
fn credit_logical(
    targets: &mut Query<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    owner: net_id::NetworkId,
    delta: i32,
) -> bool {
    for (slot_owner, player, mut counter) in targets {
        if ***slot_owner == *owner || player.is_some_and(|p| **p == *owner) {
            counter.add(delta);
            return true;
        }
    }
    false
}

// needed helper: credits the transport sender slot for unresolved remotes
fn credit_sender(
    targets: &mut Query<(
        &clicker::Owner,
        Option<&clicker::PlayerNo>,
        &mut clicker::ClickCounter,
    )>,
    sender: net_id::NetworkId,
    delta: i32,
) -> bool {
    for (slot_owner, _, mut counter) in targets {
        if ***slot_owner == *sender {
            counter.add(delta);
            return true;
        }
    }
    false
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
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(net_id::NetworkId(99));
        let sender = net_id::NetworkId::from_peer("peer");
        let target = app
            .world_mut()
            .spawn((clicker::Owner(sender), clicker::ClickCounter::default()))
            .id();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: clicker::CursorMsg::Click {
                    owner: sender,
                    delta: 3,
                },
            });
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: clicker::CursorMsg::Click {
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
        assert!(
            app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .is_empty()
        );
    }

    #[test]
    fn test_sender_credited_owner_advisory() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(net_id::NetworkId(99));
        let sender = net_id::NetworkId::from_peer("peer");
        let target = app
            .world_mut()
            .spawn((clicker::Owner(sender), clicker::ClickCounter::default()))
            .id();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: clicker::CursorMsg::Click {
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
    }

    #[test]
    fn test_missing_slot_goes_pending() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(net_id::NetworkId(99));
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "ghost".to_string(),
                payload: clicker::CursorMsg::Click {
                    owner: net_id::NetworkId(5),
                    delta: 2,
                },
            });
        app.add_systems(Update, apply_delta);
        app.update();
        assert_eq!(
            app.world().resource::<clicker::PendingClicks>().items.len(),
            1
        );
    }
}
