use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct TotalBoard;

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    #[cfg(feature = "dev")]
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    #[cfg(feature = "dev")]
    use derive_more::Deref;
    #[cfg(feature = "dev")]
    use std::sync::{Arc, Mutex};
    #[cfg(feature = "dev")]
    use std::time::Duration;

    use super::TotalBoard;
    #[cfg(feature = "dev")]
    use crate::clicker;
    #[cfg(feature = "dev")]
    use crate::testing;
    #[cfg(feature = "dev")]
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let entity = world.spawn(TotalBoard).id();
        assert!(world.get::<TotalBoard>(entity).is_some());
    }

    #[cfg(feature = "dev")]
    const PREVIEW_X: i32 = 40;
    #[cfg(feature = "dev")]
    const PREVIEW_Y: i32 = 80;
    #[cfg(feature = "dev")]
    const CLIP_SECS: u64 = 24;
    #[cfg(feature = "dev")]
    const RUN_SECS: f32 = 26.0;
    #[cfg(feature = "dev")]
    const FAST_INTERVAL_SECS: f64 = 0.25;

    #[cfg(feature = "dev")]
    #[test]
    #[ignore = "headed window: run via lens (dev feature auto-enabled)"]
    fn total_board_ui_png_preview() {
        if no_display() {
            return;
        }
        let shot = shot_path();
        let _ = std::fs::remove_file(&shot);
        let mut app = App::new();
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::GlobalCounter(1_234_567));
        app.add_plugins(testing::UiTestPlugin {
            title: "total_board".to_owned(),
            visible: false,
        });
        app.add_systems(Startup, show_board);
        app.add_systems(Update, clicker::bevy_systems::update_total_board);
        app.add_systems(Update, capture_png);
        app.insert_resource(ShotClock {
            frames: 0,
            saved: false,
            start: None,
        });
        app.run();
        assert!(shot.exists());
        println!("PREVIEW_ARTIFACT={}", shot.display());
    }

    #[cfg(feature = "dev")]
    #[test]
    #[ignore = "headed recording: run via lens (dev feature auto-enabled)"]
    fn total_board_ui_mp4_preview() {
        if no_display() {
            return;
        }
        let mp4 = mp4_path();
        let _ = std::fs::remove_file(&mp4);
        let driver = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(3));
            testing::place_window("total_board", PREVIEW_X, PREVIEW_Y)?;
            let child = testing::start_record_at(CLIP_SECS, &mp4_path(), PREVIEW_X, PREVIEW_Y)
                .ok_or_else(|| "ffmpeg did not start".to_string())?;
            for _ in 0..4 {
                testing::drive_cursor("total_board")?;
                std::thread::sleep(Duration::from_secs(2));
            }
            testing::finish_record(child, &mp4_path())
                .ok_or_else(|| "recording produced no file".to_string())?;
            Ok::<(), String>(())
        });
        let mut app = App::new();
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(roster::Roster::default());
        app.add_plugins(testing::UiTestPlugin {
            title: "total_board".to_owned(),
            visible: true,
        });
        app.add_systems(Startup, setup_scenario);
        app.add_systems(Startup, clicker::bevy_systems::spawn_cursor);
        app.add_systems(Update, clicker::bevy_systems::detect_click);
        app.add_systems(Update, clicker::bevy_systems::follow_mouse);
        app.add_systems(Update, clicker::bevy_systems::update_cursor_label);
        app.add_systems(Update, clicker::bevy_systems::emit_flash);
        app.add_systems(Update, clicker::bevy_systems::animate_flash);
        app.add_systems(Update, clicker::bevy_systems::update_total_board);
        app.add_systems(Update, script_fast_clicks);
        app.add_systems(Update, record_final);
        let stats = Arc::new(Mutex::new(FinalStats::default()));
        app.insert_resource(StatsOut(stats.clone()));
        app.run();
        assert!(driver.join().is_ok_and(|result| result.is_ok()));
        assert!(mp4.exists());
        let stats = stats.lock().unwrap();
        assert!(stats.global > 0, "player should have clicked");
        tracing::info!("scenario final total={}", stats.global);
        let (leading, significant, suffix) = clicker::odometer_formatter(stats.global);
        assert_eq!(stats.board, format!("{leading}{significant}{suffix}"));
        println!("PREVIEW_ARTIFACT={}", mp4.display());
    }

    #[cfg(feature = "dev")]
    #[derive(Resource)]
    struct ShotClock {
        frames: u32,
        saved: bool,
        start: Option<std::time::Instant>,
    }

    // needed helper: builds the single-player board scenario
    #[cfg(feature = "dev")]
    fn setup_scenario(mut commands: Commands) {
        tracing::info!("scenario setup single fast player");
        commands.spawn(Camera2d);
        commands.spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter::default(),
        ));
        commands.spawn((
            clicker::TotalBoard,
            Text2d::new(""),
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        ));
    }

    // needed helper: hammers the player's clicks on a short timer
    #[cfg(feature = "dev")]
    fn script_fast_clicks(
        time: Res<Time>,
        own: Res<net_id::NetworkId>,
        mut targets: Query<(&clicker::Owner, &mut clicker::ClickCounter)>,
        mut global: ResMut<clicker::GlobalCounter>,
        mut last: Local<f64>,
    ) {
        let now = time.into_inner().elapsed_secs_f64();
        if now - *last < FAST_INTERVAL_SECS {
            return;
        }
        *last = now;
        let own = own.into_inner();
        for (owner, mut counter) in &mut targets {
            if **owner == *own {
                counter.increment();
                global.increment();
                tracing::debug!("scripted fast click total={}", **global);
                break;
            }
        }
    }

    // needed helper: builds the still preview with a preset total
    #[cfg(feature = "dev")]
    fn show_board(mut commands: Commands) {
        commands.spawn(Camera2d);
        commands.spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter::default(),
        ));
        commands.spawn((
            clicker::TotalBoard,
            Text2d::new(""),
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        ));
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
        if !clock.saved && clock.frames >= 15 && start.elapsed() >= Duration::from_secs(2) {
            clock.saved = true;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(shot_path()));
        }
        if clock.frames >= 25 && start.elapsed() >= Duration::from_secs(3) {
            exit.write(AppExit::Success);
        }
    }

    // needed helper: snapshots the final totals out of the app before it exits
    #[cfg(feature = "dev")]
    fn record_final(
        time: Res<Time>,
        global: Res<clicker::GlobalCounter>,
        boards: Query<&Children, With<TotalBoard>>,
        spans: Query<&TextSpan>,
        out: Res<StatsOut>,
        mut exit: MessageWriter<AppExit>,
    ) {
        let time = time.into_inner();
        if time.elapsed_secs() < RUN_SECS {
            return;
        }
        let mut board = String::new();
        for children in &boards {
            for child in children.iter() {
                if let Ok(span) = spans.get(child) {
                    board.push_str(span);
                }
            }
        }
        let stats = FinalStats {
            global: **global.into_inner(),
            board,
        };
        tracing::info!("scenario final snapshot total={}", stats.global);
        if let Ok(mut out) = out.into_inner().lock() {
            *out = stats;
        }
        exit.write(AppExit::Success);
    }

    #[cfg(feature = "dev")]
    #[derive(Debug, Default, Clone)]
    struct FinalStats {
        global: i32,
        board: String,
    }

    #[cfg(feature = "dev")]
    #[derive(Resource, Clone, Deref)]
    struct StatsOut(Arc<Mutex<FinalStats>>);

    // needed helper:
    #[cfg(feature = "dev")]
    fn no_display() -> bool {
        std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err()
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn shot_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("total_board.png")
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn mp4_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("total_board.mp4")
    }
}
