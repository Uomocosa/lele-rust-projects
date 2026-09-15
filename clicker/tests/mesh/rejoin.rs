use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::roster;

use crate::support;

fn restored_count(mesh: &mut testing::Mesh, owner: usize, player: u64) -> Option<i32> {
    let mut query = mesh.apps[owner]
        .world_mut()
        .query::<(&clicker::PlayerNo, &clicker::ClickCounter)>();
    for (numbered, counter) in query.iter(mesh.apps[owner].world()) {
        if **numbered == player {
            return Some(**counter);
        }
    }
    None
}

#[test]
fn leaver_rejoins_at_old_score() {
    let mut mesh = testing::Mesh::three();
    mesh.click(2, 7);
    for _ in 0..10 {
        mesh.step();
    }
    support::push_to(&mut mesh, 1, support::move_gossip("peer-2", 2));
    for _ in 0..3 {
        mesh.step();
    }
    assert_eq!(restored_count(&mut mesh, 0, 2), Some(7));
    mesh.apps[0]
        .world_mut()
        .resource_mut::<roster::Roster>()
        .remove_entry("alpha", [2u8; 32]);
    mesh.apps[0].update();
    mesh.apps[0].update();
    assert_eq!(restored_count(&mut mesh, 0, 2), None);
    assert_eq!(
        mesh.apps[0]
            .world()
            .resource::<clicker::ScoreTombstones>()
            .get(&2),
        Some(&7)
    );
    mesh.apps[0]
        .world_mut()
        .resource_mut::<roster::Roster>()
        .add_entry("alpha".to_string(), [2u8; 32], "peer-2".to_string());
    for _ in 0..3 {
        mesh.step();
    }
    support::push_to(&mut mesh, 1, support::move_gossip("peer-2", 2));
    for _ in 0..5 {
        mesh.step();
    }
    assert_eq!(restored_count(&mut mesh, 0, 2), Some(7));
}
