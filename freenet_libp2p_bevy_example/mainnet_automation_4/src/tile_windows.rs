use std::process::Command;

use crate::Error;
use crate::window_info;

const GAP: i32 = 8;
const SCREEN_W: i32 = 1920;
const SCREEN_H: i32 = 1040;

pub fn tile_windows(instances: &[(usize, window_info::WindowInfo)]) -> Result<(), Error> {
    let n = instances.len();
    if n == 0 {
        return Ok(());
    }
    let left_w = (SCREEN_W - GAP) / 2;
    let right_w = SCREEN_W - left_w - GAP;
    let right_x = left_w + GAP;
    let right_k = (n - 1) as i32;
    let right_h = if right_k > 0 {
        (SCREEN_H - (right_k - 1) * GAP) / right_k
    } else {
        SCREEN_H
    };

    for (slot, (_, win)) in instances.iter().enumerate() {
        let (x, y, w, h) = if slot == 0 {
            (0, 0, left_w, SCREEN_H)
        } else {
            let j = (slot - 1) as i32;
            (right_x, j * (right_h + GAP), right_w, right_h)
        };
        move_window(&win.id, x, y, w, h)?;
    }
    Ok(())
}

fn move_window(id: &str, x: i32, y: i32, w: i32, h: i32) -> Result<(), Error> {
    let geometry = format!("0,{x},{y},{w},{h}");
    let status = Command::new("wmctrl")
        .args(["-i", "-r", id, "-e", &geometry])
        .status()
        .map_err(|e| Error::Window(format!("wmctrl move/resize: {e}")))?;
    if !status.success() {
        return Err(Error::Window(format!("wmctrl could not move window {id}")));
    }
    Ok(())
}

// no test_usage necessary
