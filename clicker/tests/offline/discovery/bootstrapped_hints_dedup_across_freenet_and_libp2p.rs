use clicker_lib::discovery;

#[test]
fn bootstrapped_hints_dedup_across_freenet_and_libp2p() {
    let freenet_hint = discovery::PeerHint {
        peer_id: "peer-2".to_string(),
        addrs: vec!["/ip4/10.0.0.2/tcp/4001".to_string()],
        rooms: vec!["room-1".to_string()],
        updated_at: 100,
    };
    let libp2p_hint = discovery::PeerHint {
        peer_id: "peer-2".to_string(),
        addrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
        rooms: vec!["room-1".to_string()],
        updated_at: 110,
    };
    let merged = discovery::merge_peer_hints(vec![freenet_hint, libp2p_hint]);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].updated_at, 110);
}
