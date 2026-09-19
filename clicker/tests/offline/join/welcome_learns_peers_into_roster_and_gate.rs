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
fn welcome_learns_peers_into_roster_and_gate() {
    let mut mesh = testing::Mesh::of(3);
    mesh.route();
    testing::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "peer-9".to_string(),
            payload: clicker::CursorMsg::Welcome {
                room: "alpha".to_string(),
                peers: vec!["peer-8".to_string()],
                own_score: (net_id::NetworkId(9), 5),
                joining: Vec::new(),
            },
        },
    );
    mesh.step();
    mesh.step();
    let first = app_index(&mesh, 1);
    let gate = mesh.apps[first].world().resource::<lobby::JoinGate>();
    assert!(
        gate.expected
            .as_ref()
            .is_some_and(|set| { set.contains("peer-9") && set.contains("peer-8") }),
        "welcome unions sender plus advertised peers into expected"
    );
    let leftover: Vec<String> = mesh.apps[first]
        .world()
        .resource::<p2p::Events<clicker::CursorMsg>>()
        .iter()
        .filter_map(|event| match event {
            p2p::Event::Message { payload, .. } => match payload {
                clicker::CursorMsg::WantJoin { .. } => Some("WantJoin".to_string()),
                clicker::CursorMsg::Welcome { .. } => Some("Welcome".to_string()),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert!(
        leftover.is_empty(),
        "join handshake consumed, leaked {leftover:?}"
    );
}
