use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use clicker_lib::{clicker, lobby, testing};
use freenet_libp2p_bevy_plugin::net_id;

#[test]
fn commit_reveals_together() {
    let mut mesh = testing::Mesh::of(3);
    let joiner = &mut mesh.apps[0];
    **joiner.world_mut().resource_mut::<lobby::JoinPending>() = Some("alpha".to_string());
    let mut gate = joiner.world_mut().resource_mut::<lobby::JoinGate>();
    gate.expected = Some(BTreeSet::from(["peer-2".to_string(), "peer-3".to_string()]));
    gate.synced = vec![
        lobby::SyncedPeer("peer-2".to_string()),
        lobby::SyncedPeer("peer-3".to_string()),
    ];
    joiner
        .world_mut()
        .resource_mut::<lobby::JoinClock>()
        .clicked_at = Some(Instant::now());

    for _ in 0..3 {
        mesh.step();
    }
    assert!(
        !mesh.apps[0]
            .world()
            .resource::<lobby::JoinGate>()
            .pending
            .is_empty(),
        "joiner armed its own reveal"
    );
    for app in &mesh.apps {
        assert!(
            !app.world().resource::<lobby::JoinGate>().pending.is_empty(),
            "every client armed the shared reveal"
        );
    }

    for target in [2, 3] {
        testing::push_to(&mut mesh, target, testing::move_gossip("peer-1", 1));
    }
    for _ in 0..3 {
        mesh.step();
    }

    force_due(&mut mesh);
    for _ in 0..3 {
        mesh.step();
    }

    let joiner_owner = net_id::NetworkId::from_peer("peer-1");
    for (index, app) in mesh.apps.iter_mut().enumerate() {
        let mut players: Vec<u64> = Vec::new();
        let mut query = app.world_mut().query::<&clicker::PlayerNo>();
        for player in query.iter(app.world()) {
            if !players.contains(&**player) {
                players.push(**player);
            }
        }
        assert!(
            players.contains(&1),
            "app {index} revealed the joiner: {players:?}"
        );
        assert!(
            app.world().resource::<lobby::JoinGate>().pending.is_empty(),
            "no pending reveal left"
        );
    }
    for index in [1, 2] {
        let mut query = mesh.apps[index]
            .world_mut()
            .query::<(&clicker::Owner, Option<&clicker::PlayerNo>)>();
        let joiner_slot = query
            .iter(mesh.apps[index].world())
            .find(|(owner, _)| ***owner == joiner_owner)
            .and_then(|(_, player)| player);
        assert_eq!(
            joiner_slot.map(|no| **no),
            Some(1),
            "app {index} shows the joiner numbered after the shared reveal"
        );
    }
}

// needed helper: rewinds every reveal deadline so the test does not wait five seconds
fn force_due(mesh: &mut testing::Mesh) {
    let now = Instant::now();
    let past = now.checked_sub(Duration::from_secs(1)).unwrap_or(now);
    for app in &mut mesh.apps {
        let mut gate = app.world_mut().resource_mut::<lobby::JoinGate>();
        for entry in &mut gate.pending {
            entry.reveal_at = past;
        }
        let mut markers = app.world_mut().query::<&mut clicker::PendingReveal>();
        for mut marker in markers.iter_mut(app.world_mut()) {
            marker.reveal_at = past;
        }
    }
}
