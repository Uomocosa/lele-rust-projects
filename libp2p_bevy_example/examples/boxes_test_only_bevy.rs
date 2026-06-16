use bevy::prelude::*;
use bevy_p2p_app::{boxes, p2p, sync};
use boxes::component::Player;
use boxes::component::{InputBuffer, PlayerInput, Position, Velocity};

fn main() {
    tracing::info!("Starting test_only_bevy - Basic Bevy P2P Test");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Test Only Bevy".into(),
                resolution: (640u32, 480u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((p2p::Plugin::new(p2p::Config::coop()), sync::Plugin))
        .add_systems(Startup, setup_game)
        .add_systems(Update, print_peer_info_system)
        .run();
}

fn setup_game(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Player {
            peer_id: libp2p::PeerId::random(),
            is_local: true,
        },
        Position::new(0.0, 0.0),
        Velocity::zero(),
        PlayerInput::new(),
        InputBuffer::default(),
        Sprite {
            color: Color::srgb(0.3, 0.5, 0.9),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    tracing::info!("Game setup complete - Player spawned");
}

fn print_peer_info_system(time: Res<Time>) {
    if time.elapsed_secs() < 1.0 {
        return;
    }

    tracing::debug!("test_only_bevy is running...");
}
