use clicker_lib::testing;
use freenet_libp2p_bevy_plugin::net_id;

#[test]
fn gossip_learned_peer_visible_without_connection() {
    let mut mesh = testing::Mesh::of(3);
    let unseen = "peer-4-libp2p-id".to_string();
    let expected = *net_id::NetworkId::from_peer(&unseen);
    assert!(
        !testing::owners(&mut mesh, 0).contains(&expected),
        "precondition: peer-4 unknown"
    );
    testing::push_to(
        &mut mesh,
        1,
        testing::gossip_roster(
            "peer-2",
            4,
            &unseen,
            vec!["/ip4/127.0.0.1/tcp/4004".to_string()],
            u64::MAX,
        ),
    );
    for _ in 0..5 {
        mesh.apps[0].update();
    }
    assert!(
        testing::owners(&mut mesh, 0).contains(&expected),
        "gossip-learned peer visible within 5 ticks without PeerConnected"
    );
}
