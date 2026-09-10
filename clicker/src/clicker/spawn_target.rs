use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

pub fn spawn_target(
    commands: &mut Commands,
    owner: net_id::NetworkId,
    index: usize,
    total: usize,
    is_local: bool,
) -> Entity {
    let color = if is_local {
        Color::srgb(0.2, 0.7, 0.3)
    } else {
        clicker::color_for(owner)
    };
    let pos = clicker::pos_for(index, total);
    commands
        .spawn((
            clicker::Owner(owner),
            clicker::ClickCounter::default(),
            clicker::ClickTarget,
            Sprite::from_color(color, Vec2::splat(clicker::TARGET_SIZE)),
            Transform::from_translation(Vec3::new(pos.x, pos.y, 0.0)),
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;
    use bevy::prelude::*;
    #[cfg(feature = "dev")]
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};

    use super::spawn_target;
    use crate::clicker;
    #[cfg(feature = "dev")]
    use crate::testing;
    use freenet_libp2p_bevy_plugin::net_id;
    #[cfg(feature = "dev")]
    use std::time::Duration;

    #[cfg(feature = "dev")]
    const PREVIEW_X: i32 = 40;
    #[cfg(feature = "dev")]
    const PREVIEW_Y: i32 = 80;
    #[cfg(feature = "dev")]
    const CLIP_SECS: u64 = 10;
    #[cfg(feature = "dev")]
    const RUN_SECS: f32 = 14.0;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let entity = spawn_target(&mut commands, net_id::NetworkId(7), 0, 1, true);
        queue.apply(&mut world);

        assert!(world.get::<clicker::Owner>(entity).is_some());
        assert_eq!(
            **world.get::<clicker::Owner>(entity).unwrap(),
            net_id::NetworkId(7)
        );
        assert!(world.get::<clicker::ClickTarget>(entity).is_some());
    }

    #[cfg(feature = "dev")]
    #[test]
    #[ignore = "headed viewer: run via lens (dev feature auto-enabled)"]
    fn show_ui() {
        if no_display() {
            return;
        }
        let mut app = App::new();
        app.add_plugins(testing::UiTestPlugin {
            title: "spawn_target".to_owned(),
            visible: true,
        });
        app.add_systems(Startup, show_target);
        app.run();
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
            title: "spawn_target".to_owned(),
            visible: false,
        });
        app.add_systems(Startup, show_target);
        app.add_systems(
            Startup,
            (clicker::bevy_systems::spawn_cursor, testing::park_cursor).chain(),
        );
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

    // needed helper:
    #[cfg(feature = "dev")]
    fn show_target(mut commands: Commands) {
        commands.spawn(Camera2d);
        spawn_target(&mut commands, net_id::NetworkId(7), 0, 1, true);
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
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spawn_target.png")
    }

    #[cfg(feature = "dev")]
    #[test]
    #[ignore = "headed recording: run via lens (dev feature auto-enabled)"]
    fn ui_mp4() {
        if no_display() {
            return;
        }
        let mp4 = mp4_path();
        let _ = std::fs::remove_file(&mp4);
        let driver = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(3));
            testing::place_window("spawn_target", PREVIEW_X, PREVIEW_Y)?;
            let child = testing::start_record_at(CLIP_SECS, &mp4_path(), PREVIEW_X, PREVIEW_Y)
                .ok_or_else(|| "ffmpeg did not start".to_string())?;
            testing::drive_cursor("spawn_target")?;
            testing::finish_record(child, &mp4_path())
                .ok_or_else(|| "recording produced no file".to_string())?;
            Ok::<(), String>(())
        });
        let mut app = App::new();
        app.add_plugins(testing::UiTestPlugin {
            title: "spawn_target".to_owned(),
            visible: true,
        });
        app.add_systems(Startup, show_target);
        app.add_systems(Startup, clicker::bevy_systems::spawn_cursor);
        app.add_systems(Update, clicker::bevy_systems::follow_mouse);
        app.add_systems(Update, exit_after_run);
        app.run();
        assert!(driver.join().is_ok_and(|result| result.is_ok()));
        assert!(mp4.exists());
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn exit_after_run(time: Res<Time>, mut exit: MessageWriter<AppExit>) {
        if time.elapsed_secs() > RUN_SECS {
            exit.write(AppExit::Success);
        }
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn mp4_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spawn_target.mp4")
    }
}
