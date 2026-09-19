use bevy::prelude::*;

use crate::lobby;

pub fn show_menu(
    mut commands: Commands,
    menus: Query<Entity, With<lobby::bevy_systems::MenuRoot>>,
    rooms: Res<lobby::RoomList>,
) {
    let changed = rooms.is_changed();
    let rooms = rooms.into_inner();
    if !menus.is_empty() {
        if changed {
            for entity in &menus {
                commands.entity(entity).despawn();
            }
        }
        return;
    }
    spawn_menu(&mut commands, rooms);
}

// needed helper: builds the full menu tree from the current room list
fn spawn_menu(commands: &mut Commands, rooms: &lobby::RoomList) {
    commands
        .spawn((
            lobby::bevy_systems::MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.08)),
        ))
        .with_children(|parent| {
            parent.spawn((
                lobby::bevy_systems::MenuRoot,
                Text::new(format!("rooms ({})", rooms.entries.len())),
            ));
            for entry in &rooms.entries {
                parent
                    .spawn((
                        lobby::bevy_systems::MenuRoot,
                        lobby::bevy_systems::RoomButton(entry.name.clone()),
                        Button,
                        Node {
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.3, 0.5)),
                    ))
                    .with_children(|button| {
                        button
                            .spawn((lobby::bevy_systems::MenuRoot, Text::new(entry.name.clone())));
                    });
            }
            parent
                .spawn((
                    lobby::bevy_systems::MenuRoot,
                    lobby::bevy_systems::CreateMarker,
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.5, 0.25)),
                ))
                .with_children(|button| {
                    button.spawn((lobby::bevy_systems::MenuRoot, Text::new("create new room")));
                });
        });
}

#[cfg(test)]
mod tests {
    use super::show_menu;
    use crate::lobby;
    #[cfg(feature = "dev")]
    use crate::testing;
    use bevy::prelude::*;
    #[cfg(feature = "dev")]
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    #[cfg(feature = "dev")]
    use std::time::Duration;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut rooms = lobby::RoomList::default();
        rooms.entries.push(lobby::RoomEntry {
            name: "room-a".to_string(),
            updated_at: 3,
        });
        rooms.entries.push(lobby::RoomEntry {
            name: "room-b".to_string(),
            updated_at: 4,
        });
        app.insert_resource(rooms);
        app.add_systems(Update, show_menu);
        app.update();
        app.update();
        let nodes = app
            .world_mut()
            .query::<&lobby::bevy_systems::MenuRoot>()
            .iter(app.world())
            .count();
        assert!(nodes >= 6);
        let buttons = app
            .world_mut()
            .query::<&lobby::bevy_systems::RoomButton>()
            .iter(app.world())
            .count();
        assert_eq!(buttons, 2);
        let markers = app
            .world_mut()
            .query::<&lobby::bevy_systems::CreateMarker>()
            .iter(app.world())
            .count();
        assert_eq!(markers, 1);
    }

    #[test]
    fn test_rebuilds_on_change() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(lobby::RoomList::default());
        app.add_systems(Update, show_menu);
        app.update();
        let before = app
            .world_mut()
            .query::<&lobby::bevy_systems::MenuRoot>()
            .iter(app.world())
            .count();
        assert!(before >= 3);
        app.world_mut()
            .resource_mut::<lobby::RoomList>()
            .entries
            .push(lobby::RoomEntry {
                name: "room-c".to_string(),
                updated_at: 1,
            });
        app.update();
        app.update();
        let buttons = app
            .world_mut()
            .query::<&lobby::bevy_systems::RoomButton>()
            .iter(app.world())
            .count();
        assert_eq!(buttons, 1);
    }

    #[cfg(feature = "dev")]
    #[test]
    #[ignore = "headed window: run via lens (dev feature auto-enabled)"]
    fn lobby_menu_ui_png_preview() {
        if no_display() {
            return;
        }
        let shot = shot_path();
        let _ = std::fs::remove_file(&shot);
        let mut app = App::new();
        let mut rooms = lobby::RoomList::default();
        rooms.entries.push(lobby::RoomEntry {
            name: "room-20250101-120000".to_string(),
            updated_at: 3,
        });
        rooms.entries.push(lobby::RoomEntry {
            name: "room-20250102-120000".to_string(),
            updated_at: 4,
        });
        app.insert_resource(rooms);
        app.add_plugins(testing::UiTestPlugin {
            title: "lobby-menu".to_owned(),
            visible: false,
        });
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, show_menu);
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
    #[derive(Resource)]
    struct ShotClock {
        frames: u32,
        saved: bool,
        start: Option<std::time::Instant>,
    }

    // needed helper: spawns a camera for headed previews
    #[cfg(feature = "dev")]
    fn setup_camera(mut commands: Commands) {
        commands.spawn(Camera2d);
    }

    // needed helper: screenshots the menu after two seconds
    #[cfg(feature = "dev")]
    fn capture_png(
        mut commands: Commands,
        mut clock: ResMut<ShotClock>,
        mut exit: MessageWriter<AppExit>,
    ) {
        clock.frames = clock.frames.saturating_add(1);
        let start = *clock.start.get_or_insert_with(std::time::Instant::now);
        if !clock.saved
            && clock.frames >= 15
            && start.elapsed() >= std::time::Duration::from_secs(2)
        {
            clock.saved = true;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(shot_path()));
        }
        if clock.frames >= 25 && start.elapsed() >= std::time::Duration::from_secs(3) {
            exit.write(AppExit::Success);
        }
    }

    // needed helper: skips headed previews without a display
    #[cfg(feature = "dev")]
    fn no_display() -> bool {
        std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err()
    }

    // needed helper: menu png artifact path
    #[cfg(feature = "dev")]
    fn shot_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lobby_menu.png")
    }

    #[cfg(feature = "dev")]
    #[test]
    #[ignore = "headed recording: run via lens (dev feature auto-enabled)"]
    fn lobby_menu_ui_mp4_preview() {
        if no_display() {
            return;
        }
        let mp4 = mp4_path();
        let _ = std::fs::remove_file(&mp4);
        let driver = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(3));
            testing::place_window("lobby-menu", PREVIEW_X, PREVIEW_Y)?;
            let child = testing::start_record_at(CLIP_SECS, &mp4_path(), PREVIEW_X, PREVIEW_Y)
                .ok_or_else(|| "ffmpeg did not start".to_string())?;
            testing::drive_cursor("lobby-menu")?;
            testing::finish_record(child, &mp4_path())
                .ok_or_else(|| "recording produced no file".to_string())?;
            Ok::<(), String>(())
        });
        let mut app = App::new();
        let mut list = lobby::RoomList::default();
        list.entries.push(lobby::RoomEntry {
            name: "room-20250101-120000".to_string(),
            updated_at: 3,
        });
        app.insert_resource(list);
        app.add_plugins(testing::UiTestPlugin {
            title: "lobby-menu".to_owned(),
            visible: true,
        });
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, show_menu);
        app.add_systems(Update, exit_after_run);
        app.run();
        assert!(driver.join().is_ok_and(|result| result.is_ok()));
        assert!(mp4.exists());
        println!("PREVIEW_ARTIFACT={}", mp4.display());
    }

    #[cfg(feature = "dev")]
    const PREVIEW_X: i32 = 40;
    #[cfg(feature = "dev")]
    const PREVIEW_Y: i32 = 80;
    #[cfg(feature = "dev")]
    const CLIP_SECS: u64 = 10;
    #[cfg(feature = "dev")]
    const RUN_SECS: f32 = 14.0;

    // needed helper: ends the headed recording run
    #[cfg(feature = "dev")]
    fn exit_after_run(time: Res<Time>, mut exit: MessageWriter<AppExit>) {
        let time = time.into_inner();
        if time.elapsed_secs() > RUN_SECS {
            exit.write(AppExit::Success);
        }
    }

    // needed helper: menu mp4 artifact path
    #[cfg(feature = "dev")]
    fn mp4_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lobby_menu.mp4")
    }
}
