use bevy::prelude::*;

use crate::clicker;
use crate::lobby;

pub fn despawn_leave(
    mut commands: Commands,
    overlays: Query<Entity, With<lobby::bevy_systems::LeaveRoot>>,
    loading: Query<Entity, With<lobby::bevy_systems::LoadingRoot>>,
    players: Query<Entity, With<clicker::PlayerNo>>,
) {
    for entity in overlays.iter().chain(loading.iter()).chain(players.iter()) {
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
        let loading = app
            .world_mut()
            .spawn((lobby::bevy_systems::LoadingRoot, Text::new("joining")))
            .id();
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
        assert!(
            app.world().get_entity(loading).is_err(),
            "loading overlay leaves with the room"
        );
    }

    #[test]
    fn own_cursor_removed_for_menu() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let own = app
            .world_mut()
            .spawn((
                clicker::Owner(net_id::NetworkId(1)),
                clicker::PlayerNo(1),
                clicker::ClickCounter(7),
            ))
            .id();
        app.add_systems(Update, despawn_leave);
        app.update();
        assert!(app.world().get_entity(own).is_err());
        let remaining = app
            .world_mut()
            .query::<&clicker::PlayerNo>()
            .iter(app.world())
            .count();
        assert_eq!(remaining, 0, "menu shows the OS cursor only");
    }
}
