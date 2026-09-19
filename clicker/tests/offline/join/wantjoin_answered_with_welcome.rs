use clicker_lib::{clicker, testing};
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
fn wantjoin_answered_with_welcome() {
    let mut mesh = testing::Mesh::of(3);
    mesh.route();
    testing::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "peer-9".to_string(),
            payload: clicker::CursorMsg::WantJoin {
                room: "alpha".to_string(),
            },
        },
    );
    for app in &mut mesh.apps {
        app.update();
    }
    let first = app_index(&mesh, 1);
    let welcome = mesh.apps[first]
        .world()
        .resource::<p2p::Commands<clicker::CursorMsg>>()
        .iter()
        .filter_map(|command| {
            let p2p::Command::Send { peer_id, payload } = command else {
                return None;
            };
            let clicker::CursorMsg::Welcome { room, .. } = payload else {
                return None;
            };
            Some((peer_id.clone(), room.clone()))
        })
        .collect::<Vec<(String, String)>>();
    assert!(
        welcome
            .iter()
            .any(|(peer, room)| peer == "peer-9" && room == "alpha"),
        "joiner draws exactly one welcome, got {welcome:?}"
    );
}
