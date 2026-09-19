use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{net_id, roster};

#[test]
fn gossip_does_not_resurrect_removed_peer() {
    let mut mesh = testing::Mesh::of(3);
    let gone = *net_id::NetworkId::from_peer("peer-3");
    assert!(
        testing::owners(&mut mesh, 0).contains(&gone),
        "precondition: peer-3 visible"
    );
    mesh.apps[0]
        .world_mut()
        .resource_mut::<roster::Roster>()
        .remove_entry("alpha", [3u8; 32]);
    let dead: Vec<bevy::prelude::Entity> = {
        let mut query = mesh.apps[0]
            .world_mut()
            .query::<(bevy::prelude::Entity, &clicker::Owner)>();
        query
            .iter(mesh.apps[0].world())
            .filter(|(_, owner)| ****owner == gone)
            .map(|(entity, _)| entity)
            .collect()
    };
    for entity in dead {
        mesh.apps[0].world_mut().despawn(entity);
    }
    mesh.apps[0].update();
    assert!(
        !testing::owners(&mut mesh, 0).contains(&gone),
        "precondition: peer-3 slot despawned after removal"
    );
    testing::push_to(
        &mut mesh,
        1,
        testing::gossip_roster(
            "peer-2",
            3,
            "peer-3",
            vec!["/ip4/127.0.0.1/tcp/4003".to_string()],
            u64::MAX,
        ),
    );
    for _ in 0..5 {
        mesh.apps[0].update();
    }
    assert!(
        !testing::owners(&mut mesh, 0).contains(&gone),
        "removed peer resurrected via gossip"
    );
}
