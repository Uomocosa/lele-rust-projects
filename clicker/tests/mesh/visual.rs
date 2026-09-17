use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::net_id;
use telegram_bot::TestLog;

use crate::support;

#[test]
#[telegram_bot::telegram_notify]
fn resolved_cursors_match_owner_color() {
    let test_log = TestLog::open("resolved_cursors_match_owner_color");
    test_log.line("test started");
    let mut mesh = testing::Mesh::three();
    support::push_to(&mut mesh, 1, support::move_gossip("peer-2", 2));
    support::push_to(&mut mesh, 1, support::move_gossip("peer-3", 3));
    for _ in 0..3 {
        mesh.step();
    }
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == 1 {
            let mut seen: u32 = 0;
            let mut query = app
                .world_mut()
                .query::<(&clicker::PlayerNo, &clicker::CursorColor)>();
            for (player, color) in query.iter(app.world()) {
                let want = clicker::color_for(net_id::NetworkId(**player));
                assert_eq!(**color, want, "player {} color", **player);
                seen = seen.saturating_add(1);
            }
            assert_eq!(seen, 2, "two resolved remote cursors");
        }
    }
}

#[test]
fn all_players_present_after_gossip() {
    let mut mesh = testing::Mesh::three();
    for (peer, owner) in [("peer-1", 1), ("peer-2", 2), ("peer-3", 3)] {
        for target in [1, 2, 3] {
            support::push_to(&mut mesh, target, support::move_gossip(peer, owner));
        }
    }
    for _ in 0..3 {
        mesh.step();
    }
    let counts = mesh.counts();
    assert!(
        counts.iter().all(|c| c.p1 == 0 && c.p2 == 0 && c.p3 == 0),
        "gossip alone moves no counters: {counts:?}"
    );
    for app in &mut mesh.apps {
        let mut players: Vec<u64> = Vec::new();
        let mut query = app.world_mut().query::<&clicker::PlayerNo>();
        for player in query.iter(app.world()) {
            if !players.contains(&**player) {
                players.push(**player);
            }
        }
        assert_eq!(players.len(), 3, "three logical players per app");
    }
}

#[test]
fn movement_applies_target() {
    let mut mesh = testing::Mesh::three();
    support::push_to(&mut mesh, 1, support::move_gossip("peer-2", 2));
    for _ in 0..3 {
        mesh.step();
    }
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == 1 {
            let mut query = app
                .world_mut()
                .query::<(&clicker::Owner, &clicker::TargetPos)>();
            let mut found = false;
            for (owner, target) in query.iter(app.world()) {
                let sender = net_id::NetworkId::from_peer("peer-2");
                if ***owner == *sender {
                    assert!((target.x - 30.0).abs() < 0.001);
                    assert!((target.y - 40.0).abs() < 0.001);
                    found = true;
                }
            }
            assert!(found, "peer-2 cursor target moved");
        }
    }
}
