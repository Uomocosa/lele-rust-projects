use std::process::Command;

/// Dismiss whatever is covering the screen before a capture: the desktop environment's
/// screensaver, the X DPMS blank, and a locked session. Every step is best-effort — the
/// commands are absent on some systems and failure just means that lever does not apply here.
pub fn wake() {
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let _ = Command::new("cinnamon-screensaver-command")
        .arg("--deactivate")
        .output();
    let _ = Command::new("gnome-screensaver-command")
        .arg("--deactivate")
        .output();
    let _ = Command::new("xdg-screensaver").arg("reset").output();
    let _ = Command::new("xset")
        .args(["-display", &display, "dpms", "force", "on"])
        .output();
    let _ = Command::new("loginctl").arg("unlock-session").output();
}

#[cfg(test)]
mod tests {
    use super::wake;
    use crate::test_support;

    #[test]
    fn test_usage() {
        test_support::assert_live_display();
        wake();
    }
}
