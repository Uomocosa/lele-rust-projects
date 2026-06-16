use bevy::prelude::*;
use bevy_p2p_app::{boxes, p2p, sync};
use boxes::component::Player;
use boxes::component::{InputBuffer, PlayerInput, Position, Velocity};

fn main() {
    // cargo run --example boxes
    App::new()
        .add_plugins((p2p::Plugin::new(p2p::Config::coop()), sync::Plugin))
        .add_plugins(boxes::GamePlugin)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy P2P Platformer".into(),
                resolution: (800u32, 600u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_game)
        .run();
}

fn setup_game(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Player {
            peer_id: libp2p::PeerId::random(),
            is_local: true,
        },
        Position::new(0.0, -200.0),
        Velocity::zero(),
        PlayerInput::new(),
        InputBuffer::default(),
        Sprite {
            color: Color::srgb(0.3, 0.5, 0.9),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -200.0, 0.0),
    ));
}
