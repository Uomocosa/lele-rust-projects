use std::process::Command;

use crate::Error;
use crate::window_info;

const GAP: i32 = 8;
const SCREEN_W: i32 = 1920;
const SCREEN_H: i32 = 1040;

pub fn tile_windows(instances: &[(usize, window_info::WindowInfo)]) -> Result<(), Error> {
    let n = instances.len();
    for (slot, (_, win)) in instances.iter().enumerate() {
        let (x, y, w, h) = slot_geometry(slot, n);
        move_window(&win.id, x, y, w, h)?;
    }
    Ok(())
}

/// Pure tiling scheme: left pane = first window, right stacked panes = the rest.
fn slot_geometry(slot: usize, n: usize) -> (i32, i32, i32, i32) {
    if n == 0 {
        return (0, 0, SCREEN_W, SCREEN_H);
    }
    if slot == 0 {
        (0, 0, (SCREEN_W - GAP) / 2, SCREEN_H)
    } else {
        let k = (n - 1) as i32;
        let right_h = if k > 0 {
            (SCREEN_H - (k - 1) * GAP) / k
        } else {
            SCREEN_H
        };
        let left_w = (SCREEN_W - GAP) / 2;
        let j = (slot - 1) as i32;
        (
            left_w + GAP,
            j * (right_h + GAP),
            SCREEN_W - left_w - GAP,
            right_h,
        )
    }
}

// needed helper:
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

#[cfg(test)]
mod tests {
    use super::slot_geometry;

    #[test]
    fn test_usage() {
        let (x0, y0, w0, _h0) = slot_geometry(0, 3);
        assert_eq!((x0, y0), (0, 0));
        assert_eq!(w0, (1920 - 8) / 2);
        let (x1, _y1, w1, _h1) = slot_geometry(1, 3);
        assert!(x1 > x0);
        assert_eq!(w1, w0);
    }
}
