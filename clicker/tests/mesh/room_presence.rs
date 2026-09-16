use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{net_id, roster};

use crate::support;

fn owners(mesh: &mut testing::Mesh, index: usize) -> Vec<u64> {
    let mut query = mesh.apps[index].world_mut().query::<&clicker::Owner>();
    query
        .iter(mesh.apps[index].world())
        .map(|owner| ***owner)
        .collect()
}

#[test]
fn gossip_learned_peer_visible_without_connection() {
    let mut mesh = testing::Mesh::three();
    let unseen = "peer-4-libp2p-id".to_string();
    let expected = *net_id::NetworkId::from_peer(&unseen);
    assert!(
        !owners(&mut mesh, 0).contains(&expected),
        "precondition: peer-4 unknown"
    );
    support::push_to(
        &mut mesh,
        1,
        support::roster_gossip(
            "peer-2",
            4,
            &unseen,
            vec!["/ip4/127.0.0.1/tcp/4004".to_string()],
            u64::MAX,
        ),
    );
    for _ in 0..5 {
        mesh.apps[0].update();
    }
    assert!(
        owners(&mut mesh, 0).contains(&expected),
        "gossip-learned peer visible within 5 ticks without PeerConnected"
    );
}

#[test]
fn gossip_does_not_resurrect_removed_peer() {
    let mut mesh = testing::Mesh::three();
    let gone = *net_id::NetworkId::from_peer("peer-3");
    assert!(
        owners(&mut mesh, 0).contains(&gone),
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
        !owners(&mut mesh, 0).contains(&gone),
        "precondition: peer-3 slot despawned after removal"
    );
    support::push_to(
        &mut mesh,
        1,
        support::roster_gossip(
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
        !owners(&mut mesh, 0).contains(&gone),
        "removed peer resurrected via gossip"
    );
}
