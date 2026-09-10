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
    #[cfg(feature = "dev")]
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};

    use super::spawn_cursor;
    use crate::clicker;
    #[cfg(feature = "dev")]
    use crate::testing;

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

    #[cfg(feature = "dev")]
    #[test]
    #[ignore = "headed window: run via lens (dev feature auto-enabled)"]
    fn ui_png() {
        if no_display() {
            return;
        }
        let shot = shot_path();
        let _ = std::fs::remove_file(&shot);
        let mut app = App::new();
        app.add_plugins(testing::UiTestPlugin {
            title: "spawn_cursor".to_owned(),
            visible: false,
        });
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Startup, spawn_cursor);
        app.add_systems(Update, clicker::bevy_systems::follow_mouse);
        app.add_systems(Update, capture_png);
        app.insert_resource(ShotClock {
            frames: 0,
            saved: false,
            start: None,
        });
        app.run();
        assert!(shot.exists());
    }

    #[cfg(feature = "dev")]
    #[derive(Resource)]
    struct ShotClock {
        frames: u32,
        saved: bool,
        start: Option<std::time::Instant>,
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn spawn_camera(mut commands: Commands) {
        commands.spawn(Camera2d);
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn capture_png(
        mut commands: Commands,
        mut clock: ResMut<ShotClock>,
        mut exit: MessageWriter<AppExit>,
    ) {
        clock.frames = clock.frames.saturating_add(1);
        let start = *clock.start.get_or_insert_with(std::time::Instant::now);
        if !clock.saved && clock.frames >= 5 && start.elapsed() >= std::time::Duration::from_secs(1)
        {
            clock.saved = true;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(shot_path()));
        }
        if clock.frames >= 10 && start.elapsed() >= std::time::Duration::from_secs(2) {
            exit.write(AppExit::Success);
        }
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn no_display() -> bool {
        std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err()
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn shot_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spawn_cursor.png")
    }
}
