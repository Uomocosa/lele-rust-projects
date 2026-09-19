use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p};

#[test]
fn sync_ack_merges_labeled_slot() {
    let mut mesh = testing::Mesh::of(3);
    testing::push_to(&mut mesh, 1, testing::move_gossip("peer-2", 2));
    for _ in 0..3 {
        mesh.step();
    }
    testing::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "peer-2".to_string(),
            payload: clicker::CursorMsg::SyncAck {
                target: net_id::NetworkId(1),
                entries: vec![(net_id::NetworkId(2), 7)],
            },
        },
    );
    mesh.step();
    let counts = mesh.counts();
    assert!(
        counts.iter().any(|c| c.count(2) == 7),
        "sync ack merged to 7: {counts:?}"
    );
}
