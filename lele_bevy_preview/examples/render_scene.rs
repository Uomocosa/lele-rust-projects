use std::path::PathBuf;

use bevy::prelude::*;
use lele_bevy_preview::preview::Config;
use lele_bevy_preview::scene::Scene;
use lele_bevy_preview::scene::basic::structs::{State, Timeline};

#[derive(Component)]
struct Panel;

fn build(app: &mut App) {
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((
            Panel,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(40.0),
                left: Val::Px(40.0),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.35, 0.65)),
            children![Text::new("idle")],
        ));
    });
    app.add_systems(
        Update,
        |time: Res<Time>, mut query: Query<&mut BackgroundColor, With<Panel>>| {
            let wave = (time.elapsed_secs() * 3.0).sin();
            for mut color in query.iter_mut() {
                *color = BackgroundColor(Color::srgb(
                    0.5f32.mul_add(wave, 0.4),
                    0.35,
                    0.4f32.mul_add(-wave, 0.7),
                ));
            }
        },
    );
}

const fn noop(_world: &mut World) {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from("/tmp/opencode/scene-out"), PathBuf::from);
    let scene = Scene {
        name: String::from("panel"),
        kind: lele_bevy_preview::scene::Kind::System,
        build,
        states: vec![
            State {
                label: String::from("idle"),
                apply: noop,
            },
            State {
                label: String::from("warm"),
                apply: noop,
            },
        ],
        timeline: Some(Timeline {
            label: String::from("pulse"),
            frames: 24,
            fps: 24,
            apply: noop,
        }),
    };
    let config = Config {
        out_dir: out,
        warmup_frames: 60,
        max_capture_frames: 300,
        ..Config::default()
    };
    let report = lele_bevy_preview::run(&scene, &config, "render_scene")?;
    println!("{report}");
    for capture in &report.captures {
        println!("{} {}", capture.artifact.label, capture.path.display());
    }
    Ok(())
}
