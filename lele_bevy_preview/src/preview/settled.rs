use std::path::Path;

use bevy::image::Image;
use bevy::prelude::*;

use crate::Error;
use crate::preview;
use crate::preview::basic::constants::SETTLE_CAPTURES;

pub fn settled(
    app: &mut App,
    target: &Handle<Image>,
    out: &Path,
    budget: u32,
    min_distinct_colors: usize,
) -> Result<Image, Error> {
    let mut last: Option<Image> = None;
    for _ in 0..SETTLE_CAPTURES {
        let Ok(image) = preview::once::once(app, target.clone(), out, budget) else {
            continue;
        };
        let pixels = image.data.clone().unwrap_or_default();
        if pixels.is_empty() {
            return Err(Error::BlankFrame(out.display().to_string()));
        }
        preview::ensure_visual::ensure_visual(&pixels, out, min_distinct_colors)?;
        let stable = last
            .as_ref()
            .and_then(|previous| previous.data.clone())
            .is_some_and(|previous| previous == pixels);
        if stable {
            return Ok(image);
        }
        last = Some(image);
    }
    last.ok_or(Error::NoFrame(SETTLE_CAPTURES))
}

// no test_usage necessary
