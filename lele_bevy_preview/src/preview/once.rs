use std::path::Path;
use std::sync::mpsc;

use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};

use crate::Error;
use crate::preview::basic::constants::PUMP_FRAMES;

pub fn once(app: &mut App, target: Handle<Image>, out: &Path, budget: u32) -> Result<Image, Error> {
    prepare_destination(out)?;
    let Some(image) = request(app, target, out, budget) else {
        settle(app);
        return Err(Error::NoFrame(budget));
    };
    let pixels = image.data.clone().unwrap_or_default();
    if pixels.is_empty() {
        return Err(Error::BlankFrame(out.display().to_string()));
    }
    Ok(image)
}

// needed helper: one Screenshot request, pumped until its observer reports a frame
fn request(app: &mut App, target: Handle<Image>, out: &Path, budget: u32) -> Option<Image> {
    let (sender, receiver) = mpsc::channel::<Image>();
    let destination = out.to_path_buf();
    app.world_mut()
        .spawn(Screenshot::image(target))
        .observe(move |event: On<ScreenshotCaptured>| {
            let _ = sender.send(event.image.clone());
        })
        .observe(bevy::render::view::screenshot::save_to_disk(destination));
    let frames = budget.min(PUMP_FRAMES);
    for _ in 0..frames {
        app.update();
        if let Ok(image) = receiver.try_recv() {
            return Some(image);
        }
    }
    None
}

// needed helper: a screenshot can be refused while another is in flight
fn settle(app: &mut App) {
    for _ in 0..PUMP_FRAMES {
        app.update();
    }
}

// needed helper: a stale file from a previous run would make a failed capture look like a success
fn prepare_destination(out: &Path) -> Result<(), Error> {
    let parent = out.parent().map(Path::to_path_buf);
    if let Some(parent) = parent {
        std::fs::create_dir_all(&parent).map_err(|error| Error::Io(error.to_string()))?;
    }
    let _ = std::fs::remove_file(out);
    Ok(())
}
// no test_usage necessary
