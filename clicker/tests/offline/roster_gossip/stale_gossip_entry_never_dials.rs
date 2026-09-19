use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::p2p;

fn dialed_peer_ids(mesh: &testing::Mesh, owner: u64) -> Vec<String> {
    for app in &mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == owner {
            return app
                .world()
                .resource::<p2p::Commands<clicker::CursorMsg>>()
                .iter()
                .filter_map(|cmd| match cmd {
                    p2p::Command::Dial { peer_id, .. } if !peer_id.is_empty() => {
                        Some(peer_id.clone())
                    }
                    _ => None,
                })
                .collect();
        }
    }
    Vec::new()
}

#[test]
fn stale_gossip_entry_never_dials() {
    let mut mesh = testing::Mesh::of(3);
    testing::push_to(
        &mut mesh,
        1,
        testing::gossip_roster("peer-2", 3, "ghost-id", vec![], 1),
    );
    for _ in 0..3 {
        mesh.step();
    }
    let dials = dialed_peer_ids(&mesh, 1);
    assert!(
        !dials.contains(&"ghost-id".to_string()),
        "stale entry dialed: {dials:?}"
    );
}
