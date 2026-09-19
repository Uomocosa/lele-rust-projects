use std::process::Command;

/// # Errors
/// Returns an error if the cursor cannot be moved or clicked.
pub fn mouse_click_at(wid: &str, x: f64, y: f64) -> Result<(), String> {
    let raised = Command::new("xdotool")
        .args(["windowactivate", wid])
        .status()
        .map_err(|err| format!("xdotool windowactivate: {err}"))?;
    if !raised.success() {
        return Err("xdotool windowactivate failed".to_string());
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    let moved = Command::new("xdotool")
        .args([
            "mousemove",
            "--window",
            wid,
            &format!("{x:.0}"),
            &format!("{y:.0}"),
        ])
        .status()
        .map_err(|err| format!("xdotool mousemove: {err}"))?;
    if !moved.success() {
        return Err("xdotool mousemove failed".to_string());
    }
    let clicked = Command::new("xdotool")
        .args(["click", "1"])
        .status()
        .map_err(|err| format!("xdotool click failed: {err}"))?;
    if clicked.success() {
        Ok(())
    } else {
        Err("xdotool click failed".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::mouse_click_at;

    #[test]
    fn test_usage() {
        assert!(mouse_click_at("no-such-wid-xyz", 10.0, 10.0).is_err());
    }
}
