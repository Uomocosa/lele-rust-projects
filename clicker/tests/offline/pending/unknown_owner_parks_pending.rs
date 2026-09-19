use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p};

#[test]
fn unknown_owner_parks_pending() {
    let mut mesh = testing::Mesh::of(3);
    testing::push_to(
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
