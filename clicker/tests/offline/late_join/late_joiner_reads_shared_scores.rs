use bevy::prelude::*;
use clicker_lib::{clicker, constants, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p};

fn restored_count(app: &mut App, player: u64) -> Option<i32> {
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
fn late_joiner_reads_shared_scores() {
    let mut mesh = testing::Mesh::of(3);
    mesh.peer(1).clicks(10);
    for _ in 0..10 {
        mesh.step();
    }
    let bytes = testing::live_snapshot(&mut mesh);
    let mut late = testing::fixture(4, "alpha");
    late.world_mut()
        .resource_mut::<p2p::Events<clicker::CursorMsg>>()
        .push(p2p::Event::HistoryChunk {
            lobby: "alpha".to_string(),
            chunk: constants::SNAPSHOT_CHUNK,
            data: bytes,
        });
    for _ in 0..3 {
        late.update();
    }
    assert_eq!(
        late.world().resource::<clicker::ScoreTombstones>().get(&1),
        Some(&10)
    );
    late.world_mut().spawn((
        clicker::CursorIcon,
        clicker::Owner(net_id::NetworkId::from_peer("peer-1")),
        clicker::ClickCounter::default(),
        clicker::TargetPos(Vec2::ZERO),
        Transform::default(),
    ));
    late.world_mut()
        .resource_mut::<freenet_libp2p_bevy_plugin::roster::Roster>()
        .add_entry("alpha".to_string(), [7u8; 32], "peer-1".to_string());
    late.world_mut()
        .resource_mut::<p2p::Events<clicker::CursorMsg>>()
        .push(testing::move_gossip("peer-1", 1));
    for _ in 0..5 {
        late.update();
    }
    assert_eq!(restored_count(&mut late, 1), Some(10));
}
