use crate::Error;
use crate::scene::Scene;
use crate::scene::basic::constants::{MAX_FPS, MAX_STATES, MAX_TIMELINE_FRAMES};

pub fn check(scene: &Scene) -> Result<(), Error> {
    if scene.name.trim().is_empty() {
        return Err(Error::Scene(String::from("scene name must not be empty")));
    }
    if scene.states.is_empty() {
        return Err(Error::Scene(format!(
            "scene `{}` declares no states, so it would render nothing",
            scene.name
        )));
    }
    if scene.states.len() > MAX_STATES {
        return Err(Error::Scene(format!(
            "scene `{}` declares {} states, over the cap of {MAX_STATES}",
            scene.name,
            scene.states.len()
        )));
    }
    for state in &scene.states {
        if state.label.trim().is_empty() {
            return Err(Error::Scene(format!(
                "scene `{}` has a state with an empty label",
                scene.name
            )));
        }
    }
    check_timeline(scene)
}

// needed helper: timeline rules live apart so check stays one readable list of state rules
fn check_timeline(scene: &Scene) -> Result<(), Error> {
    let Some(timeline) = scene.timeline.as_ref() else {
        return Ok(());
    };
    if timeline.frames == 0 {
        return Err(Error::Scene(format!(
            "scene `{}` timeline `{}` declares zero frames",
            scene.name, timeline.label
        )));
    }
    if timeline.frames > MAX_TIMELINE_FRAMES {
        return Err(Error::Scene(format!(
            "scene `{}` timeline `{}` declares {} frames, over the cap of {MAX_TIMELINE_FRAMES}",
            scene.name, timeline.label, timeline.frames
        )));
    }
    if timeline.fps == 0 || timeline.fps > MAX_FPS {
        return Err(Error::Scene(format!(
            "scene `{}` timeline `{}` fps {} is outside 1..={MAX_FPS}",
            scene.name, timeline.label, timeline.fps
        )));
    }
    Ok(())
}
// no test_usage necessary
