use std::path::PathBuf;

use bevy::prelude::*;
use lele_bevy_preview::Report;
use lele_bevy_preview::deliver::basic::enums::Status;
use lele_bevy_preview::preview::Config;
use lele_bevy_preview::run;
use lele_bevy_preview::scene::Scene;
use lele_bevy_preview::scene::basic::structs::{State, Timeline};

#[derive(Component)]
struct Card;

#[derive(Resource, Default)]
struct Count(u32);

fn build_scene(app: &mut App) {
    app.init_resource::<Count>();
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((
            Card,
            Node {
                width: bevy::ui::Val::Px(200.0),
                height: bevy::ui::Val::Px(60.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.4, 0.8)),
            children![Text::new("base")],
        ));
    });
}

fn with_count(app: &mut App) {
    app.init_resource::<Count>();
    app.add_systems(
        Update,
        |count: Res<Count>, mut query: Query<&mut Text, With<Card>>| {
            for mut text in query.iter_mut() {
                **text = format!("count {}", count.0);
            }
        },
    );
}

fn with_count_and_camera(app: &mut App) {
    build_scene(app);
    with_count(app);
    app.add_systems(
        Update,
        |time: Res<Time>, mut query: Query<&mut BackgroundColor, With<Card>>| {
            for mut color in query.iter_mut() {
                let phase = (time.elapsed_secs() * 2.0).sin();
                *color = BackgroundColor(Color::srgb(0.4f32.mul_add(phase, 0.5), 0.3, 0.6));
            }
        },
    );
}

fn grow(world: &mut World) {
    let mut count = world.resource_mut::<Count>();
    count.0 = count.0.saturating_add(1);
}

fn config(dir: &std::path::Path) -> Config {
    Config {
        out_dir: PathBuf::from(dir),
        warmup_frames: 30,
        lead_in_frames: 2,
        max_capture_frames: 240,
        ..Config::default()
    }
}

fn scene(states: Vec<State>) -> Scene {
    Scene {
        name: String::from("card"),
        kind: lele_bevy_preview::scene::Kind::System,
        build: build_scene,
        states,
        timeline: None,
    }
}

fn rest() -> State {
    State {
        label: String::from("rest"),
        apply: noop,
    }
}

fn noop(_world: &mut World) {}

#[test]
fn captures_a_png_per_declared_state() {
    let dir = tempfile::tempdir().unwrap();
    let report: Report = run(
        &scene(vec![
            rest(),
            State {
                label: String::from("after one tick"),
                apply: grow,
            },
        ]),
        &config(dir.path()),
        "real_scene",
    )
    .unwrap();
    assert_eq!(report.captures.len(), 2);
    for capture in &report.captures {
        assert!(capture.path.exists(), "{} missing", capture.path.display());
        assert!(
            std::fs::metadata(&capture.path).is_ok_and(|meta| meta.len() > 1024),
            "{} is suspiciously small",
            capture.path.display()
        );
    }
}

#[test]
fn different_states_render_different_pixels() {
    let dir = tempfile::tempdir().unwrap();
    let report = run(
        &scene(vec![
            rest(),
            State {
                label: String::from("tinted"),
                apply: tint,
            },
        ]),
        &config(dir.path()),
        "real_scene",
    )
    .unwrap();
    let first = &report.captures[0].artifact.pixel_hash;
    let second = &report.captures[1].artifact.pixel_hash;
    assert_ne!(first, second, "a visual change must change the pixel hash");
}

fn tint(world: &mut World) {
    let mut query = world.query_filtered::<&mut BackgroundColor, With<Card>>();
    for mut color in query.iter_mut(world) {
        *color = BackgroundColor(Color::srgb(0.9, 0.3, 0.1));
    }
}

#[test]
fn a_blank_scene_is_rejected_rather_than_writing_an_empty_png() {
    let dir = tempfile::tempdir().unwrap();
    let empty = Scene {
        name: String::from("empty"),
        kind: lele_bevy_preview::scene::Kind::System,
        build: noop_build,
        states: vec![rest()],
        timeline: None,
    };
    let result = run(&empty, &config(dir.path()), "real_scene");
    assert!(
        matches!(result, Err(lele_bevy_preview::Error::NoVisual { .. })),
        "a scene that renders only the clear colour must be rejected, not written as a blank png"
    );
}

fn noop_build(_app: &mut App) {}

#[test]
fn second_identical_run_is_reported_as_unchanged_and_is_not_sendable() {
    let dir = tempfile::tempdir().unwrap();
    let one = scene(vec![rest()]);
    run(&one, &config(dir.path()), "real_scene").unwrap();
    let two = scene(vec![rest()]);
    let second = run(&two, &config(dir.path()), "real_scene").unwrap();
    assert_eq!(second.captures[0].status, Status::Same);
    assert!(!lele_bevy_preview::deliver::sendable(
        second.captures[0].status
    ));
}

#[test]
fn a_changed_visual_is_reported_as_changed() {
    let dir = tempfile::tempdir().unwrap();
    run(&scene(vec![rest()]), &config(dir.path()), "real_scene").unwrap();
    let tinted = run(
        &scene(vec![State {
            label: String::from("rest"),
            apply: tint,
        }]),
        &config(dir.path()),
        "real_scene",
    )
    .unwrap();
    assert_eq!(tinted.captures[0].status, Status::Changed);
    assert!(lele_bevy_preview::deliver::sendable(
        tinted.captures[0].status
    ));
}

#[test]
fn a_timeline_renders_its_lead_in_then_one_frame_per_declared_frame() {
    let dir = tempfile::tempdir().unwrap();
    let with_timeline = Scene {
        name: String::from("pulse"),
        kind: lele_bevy_preview::scene::Kind::System,
        build: with_count_and_camera,
        states: vec![rest()],
        timeline: Some(Timeline {
            label: String::from("count_up"),
            frames: 4,
            fps: 30,
            apply: grow,
        }),
    };
    let frames =
        lele_bevy_preview::timeline::render_frames(&with_timeline, &config(dir.path())).unwrap();
    assert_eq!(frames.len(), 6, "2 lead-in frames plus the 4 declared ones");
    for frame in &frames {
        assert!(frame.0.exists(), "{} missing", frame.0.display());
    }
}

#[test]
fn frames_of_a_timeline_differ_as_the_value_grows() {
    let dir = tempfile::tempdir().unwrap();
    let with_timeline = Scene {
        name: String::from("pulse"),
        kind: lele_bevy_preview::scene::Kind::System,
        build: with_count_and_camera,
        states: vec![rest()],
        timeline: Some(Timeline {
            label: String::from("count_up"),
            frames: 3,
            fps: 30,
            apply: grow,
        }),
    };
    let frames =
        lele_bevy_preview::timeline::render_frames(&with_timeline, &config(dir.path())).unwrap();
    let hashes: Vec<String> = frames.iter().map(|frame| frame.1.clone()).collect();
    assert_ne!(
        hashes[0], hashes[1],
        "consecutive frames of a time-driven animation must differ"
    );
}
