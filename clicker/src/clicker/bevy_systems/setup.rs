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
        if !clock.saved
            && clock.frames >= 30
            && start.elapsed() >= std::time::Duration::from_secs(2)
        {
            clock.saved = true;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(shot_path()));
        }
        if clock.frames >= 60 && start.elapsed() >= std::time::Duration::from_secs(5) {
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
}
