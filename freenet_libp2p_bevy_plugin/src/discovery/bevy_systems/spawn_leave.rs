use bevy::prelude::*;

use super::super::leave_marker::LeaveMarker;
use super::super::leave_root::LeaveRoot;

pub fn spawn_leave(mut commands: Commands) {
    commands
        .spawn((
            LeaveRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                right: Val::Px(8.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
        ))
        .with_children(|parent| {
            parent.spawn((LeaveMarker, Button, Text::new("leave")));
        });
}
// no test_usage necessary
