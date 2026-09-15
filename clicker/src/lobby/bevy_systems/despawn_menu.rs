use bevy::prelude::*;

use crate::lobby;

pub fn despawn_menu(
    mut commands: Commands,
    menus: Query<Entity, With<lobby::bevy_systems::MenuRoot>>,
) {
    for entity in &menus {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::despawn_menu;
    use crate::lobby;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut()
            .spawn((lobby::bevy_systems::MenuRoot, Text::new("menu")));
        app.add_systems(Update, despawn_menu);
        app.update();
        let nodes = app
            .world_mut()
            .query::<&lobby::bevy_systems::MenuRoot>()
            .iter(app.world())
            .count();
        assert_eq!(nodes, 0);
    }
}
