use bevy::{prelude::*, window::WindowResolution};
use std::f32::consts::TAU;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AAI MCP Viewer — bevy-mcp-viewer v0.1".into(),
                resolution: WindowResolution::new(640, 480),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.08, 0.08, 0.12)))
        .add_systems(Startup, setup)
        .add_systems(Update, animate)
        .run();
}

#[derive(Component)]
struct Orbiter {
    radius: f32,
    speed: f32,
    phase: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Three colored shapes orbiting the centre
    let shapes: &[(f32, f32, f32, Color)] = &[
        (90.0, 0.8, 0.0,       Color::srgb(0.9, 0.3, 0.3)),
        (140.0, 0.5, TAU / 3.0, Color::srgb(0.3, 0.9, 0.4)),
        (60.0,  1.2, TAU * 2.0 / 3.0, Color::srgb(0.3, 0.5, 1.0)),
    ];

    for &(radius, speed, phase, color) in shapes {
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(18.0))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(radius, 0.0, 0.0),
            Orbiter { radius, speed, phase },
        ));
    }

    // Label
    commands.spawn((
        Text::new("AAI MCP Viewer\nbevy 0.19 | aai-mcp v0.1\n\nscreenshot via aai-mcp to see this"),
        TextFont { font_size: FontSize::Px(16.0), ..default() },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn animate(time: Res<Time>, mut query: Query<(&Orbiter, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (orb, mut tf) in &mut query {
        let angle = orb.phase + t * orb.speed;
        tf.translation.x = angle.cos() * orb.radius;
        tf.translation.y = angle.sin() * orb.radius;
    }
}
