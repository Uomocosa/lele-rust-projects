use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct TotalBoard;

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    #[cfg(feature = "dev")]
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    #[cfg(feature = "dev")]
    use derive_more::{Deref, DerefMut};
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
    use freenet_libp2p_bevy_plugin::net_id;

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
    const CLIP_SECS: u64 = 26;
    #[cfg(feature = "dev")]
    const RUN_SECS: f32 = 30.0;
    #[cfg(feature = "dev")]
    const FAST_INTERVAL_SECS: f64 = 0.3;
    #[cfg(feature = "dev")]
    const REMOVE_AT_SECS: f32 = 10.0;

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
            testing::drive_cursor("total_board")?;
            std::thread::sleep(Duration::from_secs(6));
            testing::drive_cursor("total_board")?;
            testing::finish_record(child, &mp4_path())
                .ok_or_else(|| "recording produced no file".to_string())?;
            Ok::<(), String>(())
        });
        let mut app = App::new();
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(FastPeer(net_id::NetworkId(7)));
        app.insert_resource(KeptTotal::default());
        app.insert_resource(clicker::GlobalCounter::default());
        app.add_plugins(testing::UiTestPlugin {
            title: "total_board".to_owned(),
            visible: true,
        });
        app.add_systems(Startup, setup_scenario);
        app.add_systems(Startup, clicker::bevy_systems::spawn_cursor);
        app.add_systems(Update, clicker::bevy_systems::follow_mouse);
        app.add_systems(Update, clicker::bevy_systems::update_cursor_label);
        app.add_systems(Update, clicker::bevy_systems::render);
        app.add_systems(Update, clicker::bevy_systems::emit_flash);
        app.add_systems(Update, clicker::bevy_systems::animate_flash);
        app.add_systems(Update, clicker::bevy_systems::update_total_board);
        app.add_systems(Update, script_fast_clicks);
        app.add_systems(Update, script_remove_fast);
        app.add_systems(Update, record_final);
        let stats = Arc::new(Mutex::new(FinalStats::default()));
        app.insert_resource(StatsOut(stats.clone()));
        app.run();
        assert!(driver.join().is_ok_and(|result| result.is_ok()));
        assert!(mp4.exists());
        let stats = stats.lock().unwrap();
        assert!(
            stats.kept > 0,
            "fast player should have clicked before leaving"
        );
        assert!(
            stats.global >= stats.kept,
            "total must be maintained after the fast player leaves"
        );
        let (leading, significant, suffix) = clicker::odometer_formatter(stats.global);
        assert_eq!(
            stats.board,
            format!("{leading}total: {significant}{suffix}")
        );
        println!("PREVIEW_ARTIFACT={}", mp4.display());
    }

    #[cfg(feature = "dev")]
    #[derive(Resource, Clone, Copy, Deref)]
    struct FastPeer(net_id::NetworkId);

    #[cfg(feature = "dev")]
    #[derive(Resource, Default, Clone, Copy, Deref, DerefMut)]
    struct KeptTotal(i32);

    #[cfg(feature = "dev")]
    #[derive(Resource)]
    struct ShotClock {
        frames: u32,
        saved: bool,
        start: Option<std::time::Instant>,
    }

    // needed helper: builds the two-player board scenario
    #[cfg(feature = "dev")]
    fn setup_scenario(mut commands: Commands, fast: Res<FastPeer>) {
        let fast = **fast.into_inner();
        commands.spawn(Camera2d);
        commands.spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter::default(),
        ));
        clicker::spawn_target(&mut commands, fast, 0, 2, false);
        commands.spawn((
            clicker::TotalBoard,
            Text2d::new(""),
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        ));
    }

    // needed helper: drives the fast player's clicks on a short timer
    #[cfg(feature = "dev")]
    fn script_fast_clicks(
        time: Res<Time>,
        fast: Res<FastPeer>,
        mut targets: Query<(&clicker::Owner, &mut clicker::ClickCounter)>,
        mut global: ResMut<clicker::GlobalCounter>,
        mut last: Local<f64>,
    ) {
        let now = time.into_inner().elapsed_secs_f64();
        if now - *last < FAST_INTERVAL_SECS {
            return;
        }
        *last = now;
        let fast = fast.into_inner();
        for (owner, mut counter) in &mut targets {
            if **owner == **fast {
                counter.increment();
                global.increment();
                break;
            }
        }
    }

    // needed helper: removes the fast player once, keeping its clicks in the total
    #[cfg(feature = "dev")]
    fn script_remove_fast(
        time: Res<Time>,
        mut commands: Commands,
        targets: Query<(Entity, &clicker::Owner)>,
        fast: Res<FastPeer>,
        global: Res<clicker::GlobalCounter>,
        mut kept: ResMut<KeptTotal>,
        mut done: Local<bool>,
    ) {
        if *done || time.into_inner().elapsed_secs() < REMOVE_AT_SECS {
            return;
        }
        *done = true;
        let fast = fast.into_inner();
        for (entity, owner) in &targets {
            if **owner == **fast {
                commands.entity(entity).despawn();
                break;
            }
        }
        **kept.into_inner() = **global.into_inner();
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
        if !clock.saved && clock.frames >= 5 && start.elapsed() >= Duration::from_secs(1) {
            clock.saved = true;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(shot_path()));
        }
        if clock.frames >= 10 && start.elapsed() >= Duration::from_secs(2) {
            exit.write(AppExit::Success);
        }
    }

    // needed helper:
    #[cfg(feature = "dev")]
    fn exit_after_run(time: Res<Time>, mut exit: MessageWriter<AppExit>) {
        if time.elapsed_secs() > RUN_SECS {
            exit.write(AppExit::Success);
        }
    }

    // needed helper: snapshots the final totals out of the app before it exits
    #[cfg(feature = "dev")]
    fn record_final(
        time: Res<Time>,
        global: Res<clicker::GlobalCounter>,
        kept: Res<KeptTotal>,
        boards: Query<&Children, With<TotalBoard>>,
        spans: Query<&TextSpan>,
        out: Res<StatsOut>,
        mut exit: MessageWriter<AppExit>,
    ) {
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
            kept: **kept.into_inner(),
            board,
        };
        if let Ok(mut out) = out.into_inner().0.lock() {
            *out = stats;
        }
        exit.write(AppExit::Success);
    }

    #[cfg(feature = "dev")]
    #[derive(Debug, Default, Clone)]
    struct FinalStats {
        global: i32,
        kept: i32,
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
