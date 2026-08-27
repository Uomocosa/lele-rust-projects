use std::process::Command;

pub fn restore_screen() {
    let _ = Command::new("xset").args(["s", "on"]).status();
    let _ = Command::new("xset").args(["+dpms"]).status();
}

// no test_usage necessary — requires a live X display; best-effort
