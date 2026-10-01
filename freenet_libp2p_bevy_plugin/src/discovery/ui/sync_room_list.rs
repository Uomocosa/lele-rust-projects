use bevy::prelude::*;

use crate::discovery;
use discovery::ui::{RoomButton, RoomList};

const ROOM_COLOR: Color = Color::srgb(0.16, 0.18, 0.22);
const CURRENT_ROOM_COLOR: Color = Color::srgb(0.18, 0.45, 0.25);

pub fn sync_room_list(
    mut commands: Commands,
    multiplayer: Res<discovery::Multiplayer>,
    list: Single<Entity, With<RoomList>>,
    mut ready: Local<bool>,
) {
    if *ready && !multiplayer.is_changed() {
        return;
    }
    *ready = true;
    let multiplayer = multiplayer.into_inner();
    let list = list.into_inner();
    let current = multiplayer.room.as_ref().map(|room| &room.name);
    commands
        .entity(list)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for (name, record) in &multiplayer.catalogue {
                let color = if current == Some(name) {
                    CURRENT_ROOM_COLOR
                } else {
                    ROOM_COLOR
                };
                let label = format!(
                    "{} ({}/{})",
                    name.as_str(),
                    record.members.len(),
                    record.capacity
                );
                parent.spawn((
                    RoomButton(name.clone()),
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(color),
                    children![Text::new(label)],
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::sync_room_list;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let mut multiplayer = discovery::Multiplayer::default();
        for name in ["alpha", "beta"] {
            multiplayer.catalogue.insert(
                discovery::id::RoomName(name.to_string()),
                discovery::id::RoomRecord {
                    capacity: 8,
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
        app.insert_resource(multiplayer);
        app.world_mut().spawn(discovery::ui::RoomList);
        app.add_systems(Update, sync_room_list);
        app.update();
        let mut buttons = app.world_mut().query::<&discovery::ui::RoomButton>();
        let mut names: Vec<String> = buttons
            .iter(app.world())
            .map(|button| button.as_str().to_string())
            .collect();
        names.sort();
        assert_eq!(names, vec!["alpha".to_string(), "beta".to_string()]);
    }
}
