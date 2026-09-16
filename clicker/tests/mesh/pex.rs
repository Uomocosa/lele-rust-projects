use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::p2p;

use crate::support;

#[test]
fn pex_ask_dropped_by_game_systems() {
    let mut mesh = testing::Mesh::three();
    support::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "peer-2".to_string(),
            payload: clicker::CursorMsg::PexAsk { want_rooms: true },
        },
    );
    for _ in 0..3 {
        mesh.step();
    }
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == 1 {
            let leftover: Vec<String> = app
                .world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .iter()
                .filter_map(|event| match event {
                    p2p::Event::Message { payload, .. } => match payload {
                        clicker::CursorMsg::PexAsk { .. } => Some("PexAsk".to_string()),
                        clicker::CursorMsg::PexResp { .. } => Some("PexResp".to_string()),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            assert!(leftover.is_empty(), "pex leaked: {leftover:?}");
        }
    }
    let counts = mesh.counts();
    assert!(counts.iter().all(|c| *c == testing::MeshCount::default()));
}

#[test]
fn pex_resp_dropped_by_game_systems() {
    let mut mesh = testing::Mesh::three();
    support::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "peer-2".to_string(),
            payload: clicker::CursorMsg::PexResp {
                peers: Vec::new(),
                rooms: Vec::new(),
            },
        },
    );
    for _ in 0..3 {
        mesh.step();
    }
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == 1 {
            let leftover: Vec<String> = app
                .world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .iter()
                .filter_map(|event| match event {
                    p2p::Event::Message { payload, .. } => match payload {
                        clicker::CursorMsg::PexAsk { .. } => Some("PexAsk".to_string()),
                        clicker::CursorMsg::PexResp { .. } => Some("PexResp".to_string()),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            assert!(leftover.is_empty(), "pex leaked: {leftover:?}");
        }
    }
}
