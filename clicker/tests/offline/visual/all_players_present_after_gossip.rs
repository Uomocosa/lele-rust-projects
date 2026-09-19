use clicker_lib::{clicker, testing};

#[test]
fn all_players_present_after_gossip() {
    let mut mesh = testing::Mesh::of(3);
    for (peer, owner) in [("peer-1", 1), ("peer-2", 2), ("peer-3", 3)] {
        for target in [1, 2, 3] {
            testing::push_to(&mut mesh, target, testing::move_gossip(peer, owner));
        }
    }
    for _ in 0..3 {
        mesh.step();
    }
    let counts = mesh.counts();
    assert!(
        counts
            .iter()
            .all(|c| c.count(1) == 0 && c.count(2) == 0 && c.count(3) == 0),
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
