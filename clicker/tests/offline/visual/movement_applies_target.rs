use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::net_id;

#[test]
fn movement_applies_target() {
    let mut mesh = testing::Mesh::of(3);
    testing::push_to(&mut mesh, 1, testing::move_gossip("peer-2", 2));
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
