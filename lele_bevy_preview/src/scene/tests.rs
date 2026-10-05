use crate::scene;
use bevy::ecs::world::World;
use bevy::prelude::App;

use crate::Error;
use crate::scene::Scene;
use crate::scene::basic::structs::{State, Timeline};

// needed helper: fixture shared by the tests in this file
fn noop(_world: &mut World) {}
// needed helper: fixture shared by the tests in this file
fn noop_build(_app: &mut App) {}

// needed helper: fixture shared by the tests in this file
fn scene() -> scene::Scene {
    scene::Scene {
        name: String::from("spawn_root"),
        build: noop_build,
        states: vec![State {
            label: String::from("empty"),
            apply: noop,
        }],
        timeline: Some(Timeline {
            label: String::from("press"),
            frames: 4,
            fps: 60,
            apply: noop,
        }),
    }
}

// needed helper: fixture shared by the tests in this file
fn spec(frames: u32, fps: u32) -> Timeline {
    Timeline {
        label: String::from("press"),
        frames,
        fps,
        apply: noop,
    }
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage() {
    assert!(scene::check::check(&scene()).is_ok());
    assert!(matches!(scene().states.first(), Some(State { .. })));
    assert!(matches!(scene().timeline, Some(Timeline { frames: 4, .. })));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_rejects_empty_states_zero_frames_and_blank_names() {
    let no_states = scene::Scene {
        states: Vec::new(),
        ..scene()
    };
    assert!(matches!(
        scene::check::check(&no_states),
        Err(Error::Scene(_))
    ));
    let zero_frames = scene::Scene {
        timeline: Some(spec(0, 60)),
        ..scene()
    };
    assert!(scene::check::check(&zero_frames).is_err());
    let unnamed = scene::Scene {
        name: String::new(),
        ..scene()
    };
    assert!(scene::check::check(&unnamed).is_err());
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_rejects_bad_labels_caps_and_fps() {
    let blank_label = scene::Scene {
        states: vec![State {
            label: String::from("  "),
            apply: noop,
        }],
        ..scene()
    };
    assert!(scene::check::check(&blank_label).is_err());
    assert!(
        scene::check::check(&Scene {
            timeline: Some(spec(2, 0)),
            ..scene()
        })
        .is_err()
    );
    assert!(
        scene::check::check(&Scene {
            timeline: Some(spec(2, 5000)),
            ..scene()
        })
        .is_err()
    );
    assert!(
        scene::check::check(&Scene {
            timeline: Some(spec(5000, 60)),
            ..scene()
        })
        .is_err()
    );
    let too_many = scene::Scene {
        states: (0..40)
            .map(|_| State {
                label: String::from("s"),
                apply: noop,
            })
            .collect(),
        ..scene()
    };
    assert!(scene::check::check(&too_many).is_err());
}
// no test_usage necessary
