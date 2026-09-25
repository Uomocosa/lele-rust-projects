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

pub fn log_tick(
    time: Res<Time>,
    username: Res<Username>,
    multiplayer: Res<discovery::Multiplayer>,
    mut accumulator: Local<f32>,
    mut query: Query<&mut Text, With<StatusText>>,
) {
    *accumulator += time.delta_secs();
    if *accumulator < 1.0 {
        return;
    }
    *accumulator = 0.0;
    let rooms: Vec<&str> = multiplayer
        .catalogue
        .keys()
        .map(|name| name.as_str())
        .collect();
    let catalogue = if rooms.is_empty() {
        "none".to_string()
    } else {
        rooms.join(",")
    };
    let room = multiplayer
        .room
        .as_ref()
        .map_or_else(|| "none".to_string(), |room| room.name.as_str().to_string());
    let members = multiplayer
        .room
        .as_ref()
        .map_or(0, |room| room.members.len());
    let connected = multiplayer.room.as_ref().map_or(0, |room| {
        room.members
            .values()
            .filter(|member| member.status == discovery::room_peers::DiscoveryStatus::Connected)
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
