use bevy::prelude::*;

use crate::lobby;

pub fn despawn_menu(
    mut commands: Commands,
    menus: Query<Entity, (With<lobby::bevy_systems::MenuRoot>, Without<ChildOf>)>,
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
        app.set_error_handler(bevy::ecs::error::panic);
        let root = app
            .world_mut()
            .spawn((lobby::bevy_systems::MenuRoot, Text::new("menu")))
            .id();
        app.world_mut().spawn((
            lobby::bevy_systems::MenuRoot,
            ChildOf(root),
            Text::new("child"),
        ));
        app.add_systems(Update, despawn_menu);
        app.update();
        let nodes = app
            .world_mut()
            .query::<&lobby::bevy_systems::MenuRoot>()
            .iter(app.world())
            .count();
        assert_eq!(nodes, 0);
    }

    #[test]
    fn nested_menu_nodes_despawn_once() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.set_error_handler(bevy::ecs::error::panic);
        let root = app
            .world_mut()
            .spawn((lobby::bevy_systems::MenuRoot, Text::new("menu")))
            .id();
        let button = app
            .world_mut()
            .spawn((
                lobby::bevy_systems::MenuRoot,
                ChildOf(root),
                Text::new("btn"),
            ))
            .id();
        app.world_mut().spawn((
            lobby::bevy_systems::MenuRoot,
            ChildOf(button),
            Text::new("label"),
        ));
        app.add_systems(Update, despawn_menu);
        app.update();
        let nodes = app
            .world_mut()
            .query::<&lobby::bevy_systems::MenuRoot>()
            .iter(app.world())
            .count();
        assert_eq!(
            nodes, 0,
            "recursive root despawn clears the whole tree once"
        );
    }
}
