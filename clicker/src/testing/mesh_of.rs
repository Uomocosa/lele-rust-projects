use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{p2p, roster};

use super::fake_dht::FakeDht;
use super::fixture;
use super::mesh::Mesh;
use super::player::Player;
use crate::clicker;

#[must_use]
pub fn of(peers: u32) -> Mesh {
    let players: Vec<Player> = (1..=peers).map(|id| Player(u64::from(id))).collect();
    let apps: Vec<App> = players
        .iter()
        .map(|player| fixture(**player, "alpha"))
        .collect();
    let mut mesh = Mesh {
        apps,
        players,
        blocks: Vec::new(),
        history: FakeDht::default(),
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
    let entries: Vec<(u8, String)> = mesh
        .players
        .iter()
        .map(|player| {
            (
                u8::try_from(**player).unwrap_or(0),
                format!("peer-{}", **player),
            )
        })
        .collect();
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        let mut members = app.world_mut().resource_mut::<roster::Roster>();
        for (id, name) in &entries {
            if u64::from(*id) == own {
                continue;
            }
            members.add_entry("alpha".to_string(), [*id; 32], name.clone());
        }
    }
}

// needed helper: announces every other peer to each app so sync requests fire
fn push_joins(mesh: &mut Mesh) {
    let names: Vec<String> = mesh
        .players
        .iter()
        .map(|player| format!("peer-{}", **player))
        .collect();
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        let own_name = format!("peer-{own}");
        let mut events = app
            .world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>();
        for name in &names {
            if name == &own_name {
                continue;
            }
            events.push(p2p::Event::PeerConnected(name.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::{p2p, roster};

    use super::of;
    use crate::clicker;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = of(3);
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

    #[test]
    fn scales_to_arbitrary_peer_counts() {
        assert_eq!(of(2).apps.len(), 2);
        assert_eq!(of(4).apps.len(), 4);
    }
}
