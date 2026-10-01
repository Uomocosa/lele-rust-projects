use bevy::prelude::*;

use crate::discovery;
use discovery::ui::{CreateRoomButton, RoomList, UiRoot};

const BUTTON_COLOR: Color = Color::srgb(0.20, 0.24, 0.32);

pub fn spawn_root(mut commands: Commands) {
    commands.spawn((
        UiRoot,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        },
        children![
            (
                CreateRoomButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_COLOR),
                children![Text::new("Create room")],
            ),
            (
                RoomList,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
            ),
        ],
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_root;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, spawn_root);
        app.update();
        let mut lists = app
            .world_mut()
            .query_filtered::<(), With<discovery::ui::RoomList>>();
        assert_eq!(lists.iter(app.world()).count(), 1);
    }
}
