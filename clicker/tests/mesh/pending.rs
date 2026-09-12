use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

use crate::support;

#[test]
fn unknown_owner_parks_pending() {
    let mut mesh = testing::Mesh::three();
    support::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "ghost".to_string(),
            payload: clicker::CursorMsg::Click {
                owner: net_id::NetworkId(9),
                delta: 4,
            },
        },
    );
    mesh.step();
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == 1 {
            let pending = app.world().resource::<clicker::PendingClicks>();
            assert_eq!(pending.items.len(), 1);
        }
    }
    let counts = mesh.counts();
    assert!(
        counts.iter().all(|c| c.global == 0),
        "nothing credited while parked: {counts:?}"
    );
}

#[test]
fn drain_labels_slot_on_join() {
    let mut mesh = testing::Mesh::three();
    support::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "ghost".to_string(),
            payload: clicker::CursorMsg::Click {
                owner: net_id::NetworkId(9),
                delta: 4,
            },
        },
    );
    mesh.step();
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == 1 {
            app.world_mut().resource_mut::<roster::Roster>().add_entry(
                "alpha".to_string(),
                [9u8; 32],
                "ghost".to_string(),
            );
            app.world_mut().spawn((
                clicker::Owner(net_id::NetworkId::from_peer("ghost")),
                clicker::ClickCounter::default(),
            ));
        }
    }
    mesh.step();
    let counts = mesh.counts();
    assert!(
        counts.iter().any(|c| c.p1 == 0 && c.global == 4),
        "drained 4 onto labeled slot: {counts:?}"
    );
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == 1 {
            let pending = app.world().resource::<clicker::PendingClicks>();
            assert_eq!(pending.items.len(), 0);
        }
    }
}

#[test]
fn sender_slot_pins_count() {
    let mut mesh = testing::Mesh::three();
    support::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "peer-2".to_string(),
            payload: clicker::CursorMsg::Click {
                owner: net_id::NetworkId(7),
                delta: 10,
            },
        },
    );
    mesh.step();
    let sender = *net_id::NetworkId::from_peer("peer-2");
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == 1 {
            assert_eq!(testing::get_count(app, sender), 10);
        }
    }
}
