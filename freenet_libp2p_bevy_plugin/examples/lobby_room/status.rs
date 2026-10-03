#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

#[derive(Resource)]
pub struct Username(pub String);

#[derive(Component)]
pub struct StatusText;

pub fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        StatusText,
        Text::new("lobby starting"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

/// Event-driven `lobby mesh` marker: logs the live link count the moment it changes.
pub fn log_mesh(snapshot: Res<discovery::Snapshot>, mut last: Local<Option<(String, usize)>>) {
    if !snapshot.is_changed() {
        return;
    }
    let Some(room) = snapshot.room.as_ref() else {
        return;
    };
    let connected = room
        .members
        .values()
        .filter(|member| member.status == discovery::LinkStatus::Connected)
        .count();
    let now = (room.name.as_str().to_string(), connected);
    if last.as_ref() == Some(&now) {
        return;
    }
    tracing::info!(
        "lobby mesh room={} connected={connected}",
        room.name.as_str()
    );
    *last = Some(now);
}

pub fn log_tick(
    time: Res<Time>,
    username: Res<Username>,
    snapshot: Res<discovery::Snapshot>,
    mut accumulator: Local<f32>,
    mut query: Query<&mut Text, With<StatusText>>,
) {
    *accumulator += time.delta_secs();
    if *accumulator < 1.0 {
        return;
    }
    *accumulator = 0.0;
    let rooms: Vec<&str> = snapshot
        .directory
        .keys()
        .map(|name| name.as_str())
        .collect();
    let catalogue = if rooms.is_empty() {
        "none".to_string()
    } else {
        rooms.join(",")
    };
    let room = snapshot
        .room
        .as_ref()
        .map_or_else(|| "none".to_string(), |room| room.name.as_str().to_string());
    let members = snapshot.room.as_ref().map_or(0, |room| room.members.len());
    let connected = snapshot.room.as_ref().map_or(0, |room| {
        room.members
            .values()
            .filter(|member| member.status == discovery::LinkStatus::Connected)
            .count()
    });
    tracing::info!(
        "lobby tick catalogue={catalogue} room={room} members={members} connected={connected}"
    );
    if let Ok(mut text) = query.single_mut() {
        *text = Text::new(format!(
            "{} room={room} members={members} connected={connected} rooms=[{catalogue}]",
            username.0
        ));
    }
}
