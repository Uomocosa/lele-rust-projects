use std::time::Instant;

use clicker_lib::{clicker, discovery, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p};

fn full_roster_gossip() -> p2p::Event<clicker::CursorMsg> {
    let mut state = discovery::RosterState::new();
    for (owner, peer) in [(2, "peer-2"), (3, "peer-3")] {
        state.insert(
            discovery::PlayerId(owner),
            discovery::PeerEntry {
                peer_id: peer.to_string(),
                addrs: vec![format!("/ip4/127.0.0.1/tcp/400{owner}")],
                updated_at: u64::MAX,
            },
        );
    }
    p2p::Event::Gossip {
        topic: "clicker/alpha/roster".to_string(),
        from: "peer-2".to_string(),
        data: bincode::serialize(&state).unwrap_or_default(),
    }
}

#[test]
fn all_peers_visible_within_one_second_hard_budget() {
    let started = Instant::now();
    let mut mesh = testing::Mesh::of(3);
    testing::push_to(&mut mesh, 1, full_roster_gossip());
    let mut ticks = 0;
    let expected = [
        *net_id::NetworkId::from_peer("peer-2"),
        *net_id::NetworkId::from_peer("peer-3"),
    ];
    while ticks < 5 {
        mesh.apps[0].update();
        ticks += 1;
        if expected
            .iter()
            .all(|id| testing::owners(&mut mesh, 0).contains(id))
        {
            break;
        }
    }
    assert!(
        expected
            .iter()
            .all(|id| testing::owners(&mut mesh, 0).contains(id)),
        "all peers must appear within 5 ticks"
    );
    assert!(
        started.elapsed().as_secs() < 1,
        "join budget is a hard 1s fail, elapsed={:?}",
        started.elapsed()
    );
}
