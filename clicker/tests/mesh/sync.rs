use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::support;

#[test]
fn connected_emits_sync_req() {
    let mesh = testing::Mesh::three();
    let mut found = false;
    for app in &mesh.apps {
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        found |= commands.iter().any(|cmd| {
            matches!(
                cmd,
                p2p::Command::Send {
                    payload: clicker::CursorMsg::SyncReq { .. },
                    ..
                }
            )
        });
    }
    assert!(found, "expected a SyncReq after PeerConnected");
}

#[test]
fn sync_ack_merges_labeled_slot() {
    let mut mesh = testing::Mesh::three();
    support::push_to(&mut mesh, 1, support::move_gossip("peer-2", 2));
    for _ in 0..3 {
        mesh.step();
    }
    support::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "peer-2".to_string(),
            payload: clicker::CursorMsg::SyncAck {
                target: net_id::NetworkId(1),
                entries: vec![(net_id::NetworkId(2), 7)],
            },
        },
    );
    mesh.step();
    let counts = mesh.counts();
    assert!(
        counts.iter().any(|c| c.p2 == 7),
        "sync ack merged to 7: {counts:?}"
    );
}

#[test]
fn sync_ack_unknown_parks() {
    let mut mesh = testing::Mesh::three();
    support::push_to(
        &mut mesh,
        1,
        p2p::Event::Message {
            from: "peer-2".to_string(),
            payload: clicker::CursorMsg::SyncAck {
                target: net_id::NetworkId(1),
                entries: vec![(net_id::NetworkId(9), 4)],
            },
        },
    );
    mesh.step();
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == 1 {
            let pending = app.world().resource::<clicker::PendingClicks>();
            assert_eq!(pending.items.len(), 1);
        }
    }
}
