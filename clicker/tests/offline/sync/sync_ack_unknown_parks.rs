use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p};

#[test]
fn sync_ack_unknown_parks() {
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
            let pending = app.world().resource::<clicker::PendingClicks>();
            assert_eq!(pending.items.len(), 1);
        }
    }
}
