use bevy::prelude::*;
use bevy::window::CursorOptions;

use crate::clicker;

pub fn spawn_cursor(
    mut commands: Commands,
    windows: Query<Entity, With<Window>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(entity) = windows.single() else {
        return;
    };
    commands.entity(entity).insert(CursorOptions {
        visible: false,
        ..default()
    });
    commands.spawn((
        clicker::CursorIcon,
        Mesh2d(meshes.add(clicker::cursor_mesh(1.18))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 9.9)),
    ));
    commands.spawn((
        clicker::CursorIcon,
        Mesh2d(meshes.add(clicker::cursor_mesh(1.0))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_cursor;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.add_systems(Startup, spawn_cursor);
        app.update();
        let count = app
            .world_mut()
            .query::<&clicker::CursorIcon>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);
    }
}
