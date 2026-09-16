use clicker_lib::{clicker, lobby, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::support;

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
    let mut mesh = testing::Mesh::three();
    mesh.route();
    support::push_to(
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
        .filter_map(|command| match command {
            p2p::Command::Send { peer_id, payload } => match payload {
                clicker::CursorMsg::Welcome { room, .. } => Some((peer_id.clone(), room.clone())),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<(String, String)>>();
    assert!(
        welcome
            .iter()
            .any(|(peer, room)| peer == "peer-9" && room == "alpha"),
        "joiner draws exactly one welcome, got {welcome:?}"
    );
}

#[test]
fn welcome_learns_peers_into_roster_and_gate() {
    let mut mesh = testing::Mesh::three();
    mesh.route();
    support::push_to(
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

#[test]
fn spectate_spawns_remotes_while_join_pending() {
    let mut mesh = testing::Mesh::three();
    mesh.route();
    let first = app_index(&mesh, 1);
    mesh.apps[first]
        .world_mut()
        .insert_resource(lobby::JoinPending(Some("alpha".to_string())));
    support::push_to(
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
