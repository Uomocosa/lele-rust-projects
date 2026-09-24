#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::create_marker::CreateMarker;
use super::super::menu_root::MenuRoot;
use super::super::room_button::RoomButton;
use super::super::room_entry::RoomEntry;
use super::super::room_list::RoomList;

pub fn show_menu(
    mut commands: Commands,
    menus: Query<Entity, (With<MenuRoot>, Without<ChildOf>)>,
    rooms: Res<RoomList>,
) {
    let changed = rooms.is_changed();
    if !menus.is_empty() {
        if changed {
            for entity in &menus {
                commands.entity(entity).despawn();
            }
        }
        return;
    }
    spawn_menu(&mut commands, &rooms);
}

// needed helper: builds the full menu tree from the current room list
fn spawn_menu(commands: &mut Commands, rooms: &RoomList) {
    commands
        .spawn((
            MenuRoot,
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
                MenuRoot,
                Text::new(format!("rooms ({})", rooms.entries.len())),
            ));
            parent
                .spawn((
                    MenuRoot,
                    CreateMarker,
                    Button,
                    Name::new("create-new-room"),
                    Node {
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.5, 0.25)),
                ))
                .with_children(|button| {
                    button.spawn((MenuRoot, Text::new("create new room")));
                });
            for entry in &rooms.entries {
                spawn_room_button(parent, entry);
            }
        });
}

// needed helper: spawns one room button row
fn spawn_room_button(parent: &mut ChildSpawnerCommands, entry: &RoomEntry) {
    parent
        .spawn((
            MenuRoot,
            RoomButton(entry.name.clone()),
            Button,
            Name::new(format!("room:{}", entry.name)),
            Node {
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.3, 0.5)),
        ))
        .with_children(|button| {
            button.spawn((MenuRoot, Text::new(entry.name.clone())));
        });
}

#[cfg(test)]
mod tests {
    use super::show_menu;
    use crate::discovery;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut rooms = discovery::RoomList::default();
        rooms.entries.push(discovery::RoomEntry {
            name: "room-a".to_string(),
            updated_at: 3,
        });
        app.insert_resource(rooms);
        app.add_systems(Update, show_menu);
        app.update();
        app.update();
        let buttons = app
            .world_mut()
            .query::<&discovery::RoomButton>()
            .iter(app.world())
            .count();
        assert_eq!(buttons, 1);
    }
}
