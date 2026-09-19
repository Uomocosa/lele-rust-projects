use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

#[test]
fn parked_absolute_drains_as_max() {
    let mut mesh = testing::Mesh::of(3);
    testing::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "peer-2".to_string(),
            payload: clicker::CursorMsg::SyncAck {
                target: net_id::NetworkId(1),
                entries: vec![(net_id::NetworkId(9), 4)],
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
                clicker::PlayerNo(9),
                clicker::ClickCounter(7),
            ));
        }
    }
    mesh.step();
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == 1 {
            let mut query = app
                .world_mut()
                .query::<(&clicker::PlayerNo, &clicker::ClickCounter)>();
            let mut found = false;
            for (player, counter) in query.iter(app.world()) {
                if **player == 9 {
                    assert_eq!(**counter, 7, "absolute drained as max, not added");
                    found = true;
                }
            }
            assert!(found, "labeled slot survived");
            let pending = app.world().resource::<clicker::PendingClicks>();
            assert_eq!(pending.items.len(), 0);
        }
    }
}
