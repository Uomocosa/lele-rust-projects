use bevy::time::TimeUpdateStrategy;
use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::roster;

fn restored_count(mesh: &mut testing::Mesh, owner: usize, player: u64) -> Option<i32> {
    let app = mesh.apps.get_mut(owner)?;
    let mut query = app
        .world_mut()
        .query::<(&clicker::PlayerNo, &clicker::ClickCounter)>();
    for (numbered, counter) in query.iter(app.world()) {
        if **numbered == player {
            return Some(**counter);
        }
    }
    None
}

#[test]
fn leaver_rejoins_at_old_score() {
    let mut mesh = testing::Mesh::of(3);
    mesh.peer(2).clicks(7);
    for _ in 0..10 {
        mesh.step();
    }
    testing::push_to(&mut mesh, 1, testing::move_gossip("peer-2", 2));
    for _ in 0..3 {
        mesh.step();
    }
    assert_eq!(restored_count(&mut mesh, 0, 2), Some(7));
    mesh.apps[0]
        .world_mut()
        .resource_mut::<roster::Roster>()
        .remove_entry("alpha", [2u8; 32]);
    mesh.apps[0]
        .world_mut()
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs(1),
        ));
    for _ in 0..70 {
        mesh.apps[0].update();
    }
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
    testing::push_to(&mut mesh, 1, testing::move_gossip("peer-2", 2));
    for _ in 0..5 {
        mesh.step();
    }
    assert_eq!(restored_count(&mut mesh, 0, 2), Some(7));
}
