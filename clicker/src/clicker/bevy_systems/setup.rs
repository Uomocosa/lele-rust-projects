use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

pub fn setup(
    mut commands: Commands,
    own: Res<net_id::NetworkId>,
    lobby: Res<clicker::ActiveLobby>,
) {
    let own = own.into_inner();
    let lobby = lobby.into_inner();
    commands.spawn(Camera2d);
    clicker::spawn_target(&mut commands, *own, 0, 1, true);
    commands.spawn((
        clicker::OwnScore,
        Text2d::new("you: 0"),
        Transform::from_translation(Vec3::new(0.0, 180.0, 1.0)),
    ));
    commands.spawn((
        clicker::GlobalScore,
        Text2d::new(format!("lobby {} global: 0", **lobby)),
        Transform::from_translation(Vec3::new(0.0, 150.0, 1.0)),
    ));
}

#[cfg(test)]
mod tests {
    use super::setup;
    use crate::clicker;
    #[cfg(feature = "dev")]
    use crate::testing;
    use bevy::prelude::*;
    #[cfg(feature = "dev")]
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
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
        let mut app = App::new();
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.add_systems(Update, setup);
        app.update();
        let scores = app.world_mut().query::<&Text2d>().iter(app.world()).count();
        assert_eq!(scores, 2);
    }

    #[cfg(feature = "dev")]
    #[test]
    #[ignore = "headed viewer: run via lens (dev feature auto-enabled)"]
    fn show_ui() {
        if no_display() {
            return;
        }
        let mut app = App::new();
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.add_plugins(testing::UiTestPlugin {
            title: "setup".to_owned(),
            visible: true,
        });
        app.add_systems(Startup, setup);
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
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.add_plugins(testing::UiTestPlugin {
            title: "setup".to_owned(),
            visible: false,
        });
        app.add_systems(Startup, setup);
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
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("setup.png")
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
            testing::place_window("setup", PREVIEW_X, PREVIEW_Y)?;
            let child = testing::start_record_at(CLIP_SECS, &mp4_path(), PREVIEW_X, PREVIEW_Y)
                .ok_or_else(|| "ffmpeg did not start".to_string())?;
            testing::drive_cursor("setup")?;
            testing::finish_record(child, &mp4_path())
                .ok_or_else(|| "recording produced no file".to_string())?;
            Ok::<(), String>(())
        });
        let mut app = App::new();
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.add_plugins(testing::UiTestPlugin {
            title: "setup".to_owned(),
            visible: true,
        });
        app.add_systems(Startup, setup);
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
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("setup.mp4")
    }
}
