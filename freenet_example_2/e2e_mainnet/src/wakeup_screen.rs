use std::process::Command;

pub fn wakeup_screen() {
    let _ = Command::new("xset").args(["s", "reset"]).status();
    let _ = Command::new("xset").args(["dpms", "force", "on"]).status();
    let _ = Command::new("xdg-screensaver").args(["reset"]).status();
    let _ = Command::new("loginctl").args(["unlock-session"]).status();
    let _ = Command::new("xset").args(["s", "off"]).status();
    let _ = Command::new("xset").args(["-dpms"]).status();
}

// no test_usage necessary — requires a live X display; best-effort
