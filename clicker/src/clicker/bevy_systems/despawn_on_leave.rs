use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::roster;

use crate::clicker;

pub fn despawn_on_leave(
    mut commands: Commands,
    roster: Res<roster::Roster>,
    query: Query<(Entity, &clicker::Owner)>,
    lobby: Res<clicker::ActiveLobby>,
    own: Res<net_id::NetworkId>,
) {
    let roster = roster.into_inner();
    let lobby = lobby.into_inner();
    let own = own.into_inner();
    let mut live = Vec::new();
    if let Some(members) = roster.get(&**lobby) {
        for peer in members.values() {
            live.push(net_id::NetworkId::from_peer(peer));
        }
    }
    for (entity, owner) in &query {
        if **owner != *own && !live.contains(&**owner) {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::despawn_on_leave;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, roster};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(roster::Roster::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter::default(),
        ));
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(9)),
            clicker::ClickCounter::default(),
        ));
        app.add_systems(Update, despawn_on_leave);
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
