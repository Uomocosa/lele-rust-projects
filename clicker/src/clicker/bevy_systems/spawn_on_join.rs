use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::roster;

use crate::clicker;

pub fn spawn_on_join(
    mut commands: Commands,
    roster: Res<roster::Roster>,
    owners: Query<&clicker::Owner>,
    lobby: Res<clicker::ActiveLobby>,
    own: Res<net_id::NetworkId>,
) {
    let roster = roster.into_inner();
    let lobby = lobby.into_inner();
    let own = own.into_inner();
    let mut known = Vec::new();
    for owner in &owners {
        known.push(**owner);
    }
    let Some(members) = roster.get(&**lobby) else {
        return;
    };
    for peer in members.values() {
        let id = net_id::NetworkId::from_peer(peer);
        if id == *own || known.contains(&id) {
            continue;
        }
        if known.len() >= clicker::LOBBY_CAP {
            tracing::warn!("lobby {} full, ignoring {}", **lobby, *id);
            continue;
        }
        known.push(id);
        tracing::info!("accounting for remote owner={} peer={peer}", *id);
        commands.spawn((clicker::Owner(id), clicker::ClickCounter::default()));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_on_join;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, roster};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(roster::Roster::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut().resource_mut::<roster::Roster>().add_entry(
            "alpha".to_string(),
            *blake3::hash(b"peer").as_bytes(),
            "peer".to_string(),
        );
        app.add_systems(Update, spawn_on_join);
        app.update();
        app.update();
        let count = app
            .world_mut()
            .query::<&clicker::Owner>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }
}
