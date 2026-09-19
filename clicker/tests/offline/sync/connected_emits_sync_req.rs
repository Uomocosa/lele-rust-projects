use clicker_lib::{clicker, testing};
use freenet_libp2p_bevy_plugin::p2p;

#[test]
fn connected_emits_sync_req() {
    let mesh = testing::Mesh::of(3);
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
