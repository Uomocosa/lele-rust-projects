use freenet_libp2p_bevy_plugin::{p2p, roster};

use super::mesh::Mesh;
use crate::clicker;
use crate::testing;

#[must_use]
pub fn three() -> Mesh {
    let mut mesh = testing::Mesh {
        apps: [
            testing::fixture(1, "alpha"),
            testing::fixture(2, "alpha"),
            testing::fixture(3, "alpha"),
        ],
        names: [
            "peer-1".to_string(),
            "peer-2".to_string(),
            "peer-3".to_string(),
        ],
        blocks: Vec::new(),
        history: testing::FakeDht::default(),
    };
    link_rosters(&mut mesh);
    push_joins(&mut mesh);
    for app in &mut mesh.apps {
        app.update();
    }
    mesh
}

// needed helper: cross-links lobby rosters so spawn_on_join sees remotes
fn link_rosters(mesh: &mut Mesh) {
    let entries: [([u8; 32], String); 3] = [
        ([1u8; 32], "peer-1".to_string()),
        ([2u8; 32], "peer-2".to_string()),
        ([3u8; 32], "peer-3".to_string()),
    ];
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        let mut roster = app.world_mut().resource_mut::<roster::Roster>();
        for (id, name) in &entries {
            if *name == format!("peer-{own}") {
                continue;
            }
            roster.add_entry("alpha".to_string(), *id, name.clone());
        }
    }
}

// needed helper: announces every peer to every app so sync requests fire
fn push_joins(mesh: &mut Mesh) {
    let all = mesh.names.clone();
    for app in &mut mesh.apps {
        let mut events = app
            .world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>();
        for name in &all {
            events.push(p2p::Event::PeerConnected(name.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::{p2p, roster};

    use super::three;
    use crate::clicker;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = three();
        for app in &mut mesh.apps {
            let members = app
                .world()
                .resource::<roster::Roster>()
                .get(&"alpha".to_string())
                .map(|m| m.values().count())
                .unwrap_or_default();
            assert_eq!(members, 2);
            assert!(
                !app.world()
                    .resource::<p2p::Commands<clicker::CursorMsg>>()
                    .is_empty()
            );
        }
        assert_eq!(testing::get_count(&mut mesh.apps[0], 1), 0);
    }
}
