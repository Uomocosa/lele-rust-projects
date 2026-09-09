use std::process::Command;

/// # Errors
/// Returns an error if `wmctrl` cannot move or activate the window.
pub fn place_window(title: &str, x: i32, y: i32) -> Result<(), String> {
    let geometry = format!("0,{x},{y},800,450");
    let status = Command::new("wmctrl")
        .args(["-r", title, "-e", &geometry])
        .status()
        .map_err(|err| format!("wmctrl move {title}: {err}"))?;
    if !status.success() {
        return Err(format!("wmctrl cannot move window {title}"));
    }
    let status = Command::new("wmctrl")
        .args(["-a", title])
        .status()
        .map_err(|err| format!("wmctrl activate {title}: {err}"))?;
    if !status.success() {
        return Err(format!("wmctrl cannot activate window {title}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::place_window;

    #[test]
    fn test_usage() {
        assert!(place_window("no-such-clicker-window-xyz", 0, 0).is_err());
    }
}
