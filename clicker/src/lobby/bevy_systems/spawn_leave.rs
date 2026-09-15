use bevy::prelude::*;

use crate::lobby;

pub fn spawn_leave(mut commands: Commands) {
    commands
        .spawn((
            lobby::bevy_systems::LeaveRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    lobby::bevy_systems::LeaveRoot,
                    lobby::bevy_systems::LeaveMarker,
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                ))
                .with_children(|button| {
                    button.spawn((lobby::bevy_systems::LeaveRoot, Text::new("leave room")));
                });
        });
}

#[cfg(test)]
mod tests {
    use super::spawn_leave;
    use crate::lobby;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, spawn_leave);
        app.update();
        let markers = app
            .world_mut()
            .query::<&lobby::bevy_systems::LeaveMarker>()
            .iter(app.world())
            .count();
        assert_eq!(markers, 1);
    }
}
