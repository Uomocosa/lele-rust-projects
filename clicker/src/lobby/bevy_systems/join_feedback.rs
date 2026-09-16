use bevy::prelude::*;

use crate::lobby;

pub fn join_feedback(
    mut commands: Commands,
    pending: Res<lobby::JoinPending>,
    assets: Option<Res<AssetServer>>,
    mut buttons: Query<(
        Entity,
        &lobby::bevy_systems::RoomButton,
        &mut BackgroundColor,
    )>,
    spinners: Query<(Entity, &mut Transform), With<lobby::bevy_systems::JoinSpinner>>,
    time: Res<Time>,
) {
    let pending = pending.into_inner();
    let time = time.into_inner();
    let Some(room) = (**pending).clone() else {
        return;
    };
    let handle: Handle<Image> = assets.map_or_else(Handle::default, |server| {
        server.load(lobby::constants::SPINNER_PATH)
    });
    for (entity, button, mut color) in &mut buttons {
        if **button == room {
            *color = BackgroundColor(Color::srgb(0.2, 0.45, 0.7));
            if spinners.is_empty() {
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        lobby::bevy_systems::JoinSpinner,
                        ImageNode::new(handle.clone()),
                        Node {
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        Transform::default(),
                    ));
                });
            }
        } else {
            *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
        }
    }
    let delta = time.delta_secs();
    for (_, mut transform) in spinners {
        transform.rotate_z(lobby::constants::SPINNER_SPEED * delta);
    }
}

#[cfg(test)]
mod tests {
    use super::join_feedback;
    use crate::lobby;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Image>();
        app.init_resource::<Time>();
        app.insert_resource(lobby::JoinPending(Some("room-a".to_string())));
        app.world_mut().spawn((
            lobby::bevy_systems::MenuRoot,
            lobby::bevy_systems::RoomButton("room-a".to_string()),
            Button,
            BackgroundColor(Color::srgb(0.15, 0.3, 0.5)),
        ));
        app.world_mut().spawn((
            lobby::bevy_systems::MenuRoot,
            lobby::bevy_systems::RoomButton("room-b".to_string()),
            Button,
            BackgroundColor(Color::srgb(0.15, 0.3, 0.5)),
        ));
        app.add_systems(Update, join_feedback);
        app.update();
        app.update();
        let mut found_gray = false;
        let mut query = app
            .world_mut()
            .query::<(&lobby::bevy_systems::RoomButton, &BackgroundColor)>();
        for (button, color) in query.iter(app.world()) {
            if **button == "room-b" && color.0 == Color::srgb(0.25, 0.25, 0.25) {
                found_gray = true;
            }
        }
        let spinners = app
            .world_mut()
            .query::<&lobby::bevy_systems::JoinSpinner>()
            .iter(app.world())
            .count();
        assert!(found_gray, "other rooms turn gray while pending");
        assert!(spinners >= 1, "spinner image attached to pending room");
    }
}
