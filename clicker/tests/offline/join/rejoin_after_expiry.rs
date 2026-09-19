use std::time::{Duration, Instant};

use clicker_lib::{clicker, lobby, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p};

#[test]
fn rejoin_after_expiry_respawns() {
    let mut mesh = testing::Mesh::of(3);
    let index = 1;
    let owner = 2;
    let target = net_id::NetworkId::from_peer("peer-1");

    {
        let mut gate = mesh.apps[index]
            .world_mut()
            .resource_mut::<lobby::JoinGate>();
        gate.arm_pending(lobby::PendingJoin {
            peer: "peer-1".to_string(),
            joiner: net_id::NetworkId(1),
            reveal_at: Instant::now()
                .checked_add(Duration::from_secs(60))
                .unwrap_or_else(Instant::now),
            fail_at: Instant::now()
                .checked_add(Duration::from_secs(90))
                .unwrap_or_else(Instant::now),
        });
    }
    for _ in 0..2 {
        mesh.step();
    }
    force_due(&mut mesh);
    for _ in 0..2 {
        mesh.step();
    }
    assert!(
        !testing::owners(&mut mesh, index).contains(&target),
        "placeholder expired without identity"
    );
    assert!(
        mesh.apps[index]
            .world()
            .resource::<lobby::JoinGate>()
            .absent
            .contains(&"peer-1".to_string()),
        "the failed peer is marked absent"
    );
    for _ in 0..3 {
        mesh.step();
    }
    assert!(
        !testing::owners(&mut mesh, index).contains(&target),
        "stale roster alone never respawns the absent peer"
    );

    testing::push_to(
        &mut mesh,
        owner,
        p2p::Event::PeerDisconnected("peer-1".to_string()),
    );
    testing::push_to(
        &mut mesh,
        owner,
        p2p::Event::PeerConnected("peer-1".to_string()),
    );
    for _ in 0..2 {
        mesh.step();
    }
    assert!(
        testing::owners(&mut mesh, index).contains(&target),
        "a real reconnect clears absence and respawns the peer"
    );

    testing::push_to(&mut mesh, owner, testing::move_gossip("peer-1", 1));
    for _ in 0..3 {
        mesh.step();
    }
    let mut query = mesh.apps[index]
        .world_mut()
        .query::<(&clicker::Owner, Option<&clicker::PlayerNo>)>();
    let slot = query
        .iter(mesh.apps[index].world())
        .find(|(owner, _)| ***owner == target)
        .and_then(|(_, player)| player);
    assert_eq!(
        slot.map(|no| **no),
        Some(1),
        "rejoined peer resolves to its number"
    );
}

// needed helper: rewinds every reveal deadline so the test does not wait for it
fn force_due(mesh: &mut testing::Mesh) {
    let now = Instant::now();
    let past = now.checked_sub(Duration::from_secs(1)).unwrap_or(now);
    for app in &mut mesh.apps {
        let mut gate = app.world_mut().resource_mut::<lobby::JoinGate>();
        for entry in &mut gate.pending {
            entry.reveal_at = past;
            entry.fail_at = past;
        }
        let mut markers = app.world_mut().query::<&mut clicker::PendingReveal>();
        for mut marker in markers.iter_mut(app.world_mut()) {
            marker.reveal_at = past;
            marker.fail_at = past;
        }
    }
}
