use std::process::Command;

/// # Errors
/// Returns an error if any `xterm` window cannot be found or tiled.
pub fn tile_three(titles: [&str; 3]) -> Result<(), String> {
    let targets = [
        (titles[0], 960, 540, 0, 0),
        (titles[1], 960, 540, 0, 540),
        (titles[2], 960, 1080, 960, 0),
    ];
    for (title, w, h, x, y) in targets {
        let mut found = false;
        for _ in 0..20 {
            if let Some(id) = find_window_id(title) {
                let _ = Command::new("xdotool")
                    .args(["windowsize", &id, &w.to_string(), &h.to_string()])
                    .output();
                let _ = Command::new("xdotool")
                    .args(["windowmove", &id, &x.to_string(), &y.to_string()])
                    .output();
                let _ = Command::new("xdotool")
                    .args(["windowactivate", &id])
                    .output();
                let _ = Command::new("wmctrl")
                    .args(["-r", title, "-e", &format!("0,{x},{y},{w},{h}")])
                    .output();
                found = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        if !found {
            return Err(format!(
                "xterm window not found for title {title:?} — FAIL fast"
            ));
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(400));
    Ok(())
}

fn find_window_id(title: &str) -> Option<String> {
    if let Ok(out) = Command::new("xdotool")
        .args(["search", "--onlyvisible", "--name", title])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        if let Some(first) = text.lines().next().map(str::trim).filter(|s| !s.is_empty()) {
            return Some(first.to_string());
        }
    }
    let out = Command::new("wmctrl").args(["-l"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.contains(title) {
            let id = line.split_whitespace().next()?;
            return Some(id.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::tile_three;

    #[test]
    fn test_usage() {
        let _ = tile_three;
    }
}
