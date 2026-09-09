use std::process::Command;

const WAYPOINTS: [(i32, i32); 5] = [(100, 100), (700, 120), (650, 350), (120, 330), (400, 225)];

/// # Errors
/// Returns an error if the window cannot be found or the synthetic mouse
/// cannot be moved or clicked.
pub fn drive_cursor(title: &str) -> Result<(), String> {
    let wid = window_id(title)?;
    for (x, y) in WAYPOINTS {
        mouse_move(&wid, x, y)?;
        std::thread::sleep(std::time::Duration::from_millis(400));
    }
    mouse_click(&wid)?;
    std::thread::sleep(std::time::Duration::from_millis(400));
    mouse_move(&wid, WAYPOINTS[0].0, WAYPOINTS[0].1)?;
    Ok(())
}

// needed helper: resolves the X window id by exact title match
fn window_id(title: &str) -> Result<String, String> {
    let pattern = format!("^{title}$");
    let output = Command::new("xdotool")
        .args(["search", "--onlyvisible", "--name", &pattern])
        .output()
        .map_err(|err| format!("xdotool search {title}: {err}"))?;
    if !output.status.success() {
        return Err(format!("xdotool cannot find window {title}"));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .next()
        .map(str::to_string)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| format!("xdotool found no id for {title}"))
}

// needed helper: moves the real cursor to window-relative coordinates
fn mouse_move(wid: &str, x: i32, y: i32) -> Result<(), String> {
    let status = Command::new("xdotool")
        .args(["mousemove", "--window", wid, &x.to_string(), &y.to_string()])
        .status()
        .map_err(|err| format!("xdotool mousemove: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("xdotool mousemove failed".to_string())
    }
}

// needed helper: sends a left click to the window
fn mouse_click(wid: &str) -> Result<(), String> {
    let status = Command::new("xdotool")
        .args(["click", "--window", wid, "1"])
        .status()
        .map_err(|err| format!("xdotool click: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("xdotool click failed".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::drive_cursor;

    #[test]
    fn test_usage() {
        assert!(drive_cursor("no-such-clicker-window-xyz").is_err());
    }
}
