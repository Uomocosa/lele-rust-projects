use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p};

#[test]
fn sender_slot_pins_count() {
    let mut mesh = testing::Mesh::of(3);
    testing::push_to(
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
