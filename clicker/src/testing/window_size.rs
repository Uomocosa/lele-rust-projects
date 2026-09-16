use std::process::Command;

/// # Errors
/// Returns an error if the window geometry cannot be read or parsed.
pub(crate) fn window_size(wid: &str) -> Result<(i32, i32), String> {
    let output = Command::new("xdotool")
        .args(["getwindowgeometry", "--shell", wid])
        .output()
        .map_err(|err| format!("xdotool geometry {wid}: {err}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut width = None;
    let mut height = None;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("WIDTH=") {
            width = value.parse::<i32>().ok();
        }
        if let Some(value) = line.strip_prefix("HEIGHT=") {
            height = value.parse::<i32>().ok();
        }
    }
    match (width, height) {
        (Some(w), Some(h)) => Ok((w, h)),
        _ => Err(format!("xdotool geometry unparseable for {wid}")),
    }
}

#[cfg(test)]
mod tests {
    use super::window_size;

    #[test]
    fn test_usage() {
        assert!(window_size("no-such-wid-xyz").is_err());
    }
}
