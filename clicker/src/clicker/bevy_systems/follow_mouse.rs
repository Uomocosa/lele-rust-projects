use bevy::prelude::*;

use crate::clicker;

pub fn follow_mouse(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut cursors: Query<&mut Transform, With<clicker::CursorIcon>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(position) = window.cursor_position() else {
        return;
    };
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    let Ok(world) = camera.viewport_to_world_2d(transform, position) else {
        return;
    };
    for mut cursor in &mut cursors {
        cursor.translation.x = world.x;
        cursor.translation.y = world.y;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::follow_mouse;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let cursor = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                Transform::from_translation(Vec3::new(9999.0, 9999.0, 10.0)),
            ))
            .id();
        app.add_systems(Update, follow_mouse);
        app.update();
        let kept = app.world().get::<Transform>(cursor).unwrap().translation;
        assert_eq!(kept.x, 9999.0);
        app.world_mut().spawn(Window::default());
        app.update();
        let kept = app.world().get::<Transform>(cursor).unwrap().translation;
        assert_eq!(kept.x, 9999.0);
    }
}
