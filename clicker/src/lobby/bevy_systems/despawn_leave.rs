use bevy::prelude::*;

use crate::clicker;
use crate::lobby;

pub fn despawn_leave(
    mut commands: Commands,
    overlays: Query<Entity, With<lobby::bevy_systems::LeaveRoot>>,
    players: Query<Entity, With<clicker::PlayerNo>>,
) {
    for entity in overlays.iter().chain(players.iter()) {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::despawn_leave;
    use crate::clicker;
    use crate::lobby;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut()
            .spawn((lobby::bevy_systems::LeaveRoot, Text::new("leave")));
        let player = app
            .world_mut()
            .spawn((
                clicker::Owner(net_id::NetworkId(2)),
                clicker::PlayerNo(2),
                clicker::ClickCounter::default(),
            ))
            .id();
        app.add_systems(Update, despawn_leave);
        app.update();
        let nodes = app
            .world_mut()
            .query::<&lobby::bevy_systems::LeaveRoot>()
            .iter(app.world())
            .count();
        assert_eq!(nodes, 0);
        assert!(app.world().get_entity(player).is_err());
    }
}
