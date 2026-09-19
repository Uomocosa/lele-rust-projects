use clicker_lib::{clicker, lobby, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p};

fn app_index(mesh: &testing::Mesh, owner: u64) -> usize {
    mesh.apps
        .iter()
        .position(|app| {
            app.world().resource::<clicker::InstanceInfo>().own_id == net_id::NetworkId(owner)
        })
        .unwrap_or_default()
}

#[test]
fn spectate_spawns_remotes_while_join_pending() {
    let mut mesh = testing::Mesh::of(3);
    mesh.route();
    let first = app_index(&mesh, 1);
    mesh.apps[first]
        .world_mut()
        .insert_resource(lobby::JoinPending(Some("alpha".to_string())));
    testing::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "peer-9".to_string(),
            payload: clicker::CursorMsg::Welcome {
                room: "alpha".to_string(),
                peers: Vec::new(),
                own_score: (net_id::NetworkId(9), 0),
                joining: Vec::new(),
            },
        },
    );
    for _ in 0..3 {
        mesh.step();
    }
    let spawned = mesh.apps[first]
        .world_mut()
        .query::<&clicker::Owner>()
        .iter(mesh.apps[first].world())
        .any(|owner| ***owner == *net_id::NetworkId::from_peer("peer-9"));
    assert!(
        spawned,
        "spectating joiner renders the room before leaving loading"
    );
    assert!(
        mesh.apps[first]
            .world()
            .resource::<lobby::JoinPending>()
            .is_some(),
        "spectating does not clear loading early"
    );
}
