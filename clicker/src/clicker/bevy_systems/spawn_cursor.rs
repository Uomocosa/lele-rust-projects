use bevy::prelude::*;
use bevy::window::CursorOptions;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

pub fn spawn_cursor(
    mut commands: Commands,
    windows: Query<Entity, With<Window>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    own: Res<net_id::NetworkId>,
) {
    let Ok(entity) = windows.single() else {
        return;
    };
    let id = *own.into_inner();
    tracing::info!("spawn cursor owner={}", *id);
    tracing::info!(
        "cursor color player={} hue={:.1}",
        *id,
        clicker::hue_for(id)
    );
    let spot = clicker::spawn_spot(id);
    commands.entity(entity).insert(CursorOptions {
        visible: false,
        ..default()
    });
    commands.spawn((
        clicker::CursorIcon,
        Mesh2d(meshes.add(clicker::cursor_mesh(1.18))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_translation(Vec3::new(spot.x, spot.y, 9.9)),
    ));
    let fill_color = clicker::color_for(id);
    let fill = commands
        .spawn((
            clicker::CursorIcon,
            clicker::Owner(id),
            clicker::PlayerNo(*id),
            clicker::ClickCounter::default(),
            clicker::CursorColor(fill_color),
            Mesh2d(meshes.add(clicker::cursor_mesh(1.0))),
            MeshMaterial2d(materials.add(fill_color)),
            Transform::from_translation(Vec3::new(spot.x, spot.y, 10.0)),
        ))
        .id();
    commands.entity(fill).with_children(|parent| {
        parent.spawn((
            clicker::CursorLabel,
            Text2d::new(clicker::math_formatter(0)),
            Transform::from_translation(Vec3::new(-17.0, 34.0, 0.5)),
        ));
    });
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
    use bevy::ecs::system::RunSystemOnce;
    #[cfg(feature = "dev")]
    use derive_more::Deref;
    use freenet_libp2p_bevy_plugin::net_id;

    // needed helper: enables debug logs for this test only
    fn test_logging(app: &mut App) {
        app.add_plugins(bevy::log::LogPlugin {
            level: bevy::log::Level::DEBUG,
            filter: "info,clicker=debug,clicker_lib=debug".to_string(),
            ..default()
        });
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        test_logging(&mut app);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(net_id::NetworkId(7));
        app.add_systems(Startup, spawn_cursor);
        app.update();
        let count = app
            .world_mut()
            .query::<&clicker::CursorIcon>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);
        app.world_mut().spawn(Window::default());
        assert!(app.world_mut().run_system_once(spawn_cursor).is_ok());
        app.update();
        let count = app
            .world_mut()
            .query::<&clicker::CursorIcon>()
            .iter(app.world())
            .count();
        assert_eq!(count, 2);
        let labels = app
            .world_mut()
            .query_filtered::<&Text2d, With<clicker::CursorLabel>>()
            .iter(app.world())
            .count();
        assert_eq!(labels, 1);
    }

    #[cfg(feature = "dev")]
    #[test]
    #[ignore = "headed window: run via lens (dev feature auto-enabled)"]
    fn spawn_cursor_ui_png_preview() {
        if no_display() {
            return;
        }
        let shot = run_png_preview("spawn_cursor", 1_234, "spawn_cursor.png");
        assert!(shot.exists());
        println!("PREVIEW_ARTIFACT={}", shot.display());
    }

    #[cfg(feature = "dev")]
    #[test]
    #[ignore = "headed window: run via lens (dev feature auto-enabled)"]
    fn spawn_cursor_single_digit_ui_png_preview() {
        if no_display() {
            return;
        }
        let shot = run_png_preview("spawn_cursor_single", 1, "spawn_cursor_single.png");
        assert!(shot.exists());
        println!("PREVIEW_ARTIFACT={}", shot.display());
    }

    #[cfg(feature = "dev")]
    #[test]
    #[ignore = "headed window: run via lens (dev feature auto-enabled)"]
    fn spawn_cursor_triple_digit_ui_png_preview() {
        if no_display() {
            return;
        }
        let shot = run_png_preview("spawn_cursor_triple", 999, "spawn_cursor_triple.png");
        assert!(shot.exists());
        println!("PREVIEW_ARTIFACT={}", shot.display());
    }

    #[cfg(feature = "dev")]
    #[derive(Resource, Clone, Copy, Deref)]
    struct PreviewCount(i32);

    #[cfg(feature = "dev")]
    #[derive(Resource, Clone, Deref)]
    struct ShotFile(std::path::PathBuf);

    #[cfg(feature = "dev")]
    #[derive(Resource)]
    struct ShotClock {
        frames: u32,
        saved: bool,
        start: Option<std::time::Instant>,
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn run_png_preview(title: &str, count: i32, file: &str) -> std::path::PathBuf {
        let shot = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(file);
        let _ = std::fs::remove_file(&shot);
        let mut app = App::new();
        app.insert_resource(net_id::NetworkId(7));
        app.insert_resource(PreviewCount(count));
        app.insert_resource(ShotFile(shot.clone()));
        app.insert_resource(ShotClock {
            frames: 0,
            saved: false,
            start: None,
        });
        app.add_plugins(testing::UiTestPlugin {
            title: title.to_owned(),
            visible: false,
        });
        app.add_systems(Startup, show_preview);
        app.add_systems(
            Startup,
            (
                clicker::bevy_systems::spawn_cursor,
                testing::bevy_systems::park_cursor,
            )
                .chain(),
        );
        app.add_systems(Update, clicker::bevy_systems::follow_mouse);
        app.add_systems(Update, clicker::bevy_systems::update_cursor_label);
        app.add_systems(Update, capture_png);
        app.run();
        shot
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn show_preview(mut commands: Commands, count: Res<PreviewCount>) {
        let count = **count.into_inner();
        commands.spawn(Camera2d);
        commands.spawn((
            clicker::Owner(net_id::NetworkId(7)),
            clicker::ClickCounter(count),
        ));
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn capture_png(
        mut commands: Commands,
        mut clock: ResMut<ShotClock>,
        shot: Res<ShotFile>,
        mut exit: MessageWriter<AppExit>,
    ) {
        clock.frames = clock.frames.saturating_add(1);
        let start = *clock.start.get_or_insert_with(std::time::Instant::now);
        let path: &std::path::PathBuf = shot.into_inner();
        if !clock.saved
            && clock.frames >= 15
            && start.elapsed() >= std::time::Duration::from_secs(2)
        {
            clock.saved = true;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path.clone()));
        }
        if clock.frames >= 25 && start.elapsed() >= std::time::Duration::from_secs(3) {
            exit.write(AppExit::Success);
        }
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn no_display() -> bool {
        std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err()
    }
}
