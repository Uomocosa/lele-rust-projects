use clicker_lib::testing;
use freenet_libp2p_bevy_plugin::net_id;

#[test]
fn no_self_ghost_after_join() {
    let mut mesh = testing::Mesh::of(3);
    for _ in 0..5 {
        mesh.step();
    }
    for index in 0..mesh.apps.len() {
        let own_peer = format!("peer-{}", index.saturating_add(1));
        let self_id = *net_id::NetworkId::from_peer(&own_peer);
        let owners = testing::owners(&mut mesh, index);
        assert!(
            !owners.contains(&self_id),
            "app {index} spawned a gray ghost for its own peer id: {owners:?}"
        );
    }
}
