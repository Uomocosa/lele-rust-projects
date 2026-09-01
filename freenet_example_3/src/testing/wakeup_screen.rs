use std::process::Command;

pub fn wakeup_screen() {
    let _ = Command::new("xdotool")
        .args(["mousemove", "1", "1"])
        .output();
    let _ = Command::new("xdotool")
        .args(["mousemove", "2", "2"])
        .output();
    let _ = Command::new("xset").args(["s", "reset"]).output();
    let _ = Command::new("xset").args(["dpms", "force", "on"]).output();
}

#[cfg(test)]
mod tests {
    use super::wakeup_screen;

    #[test]
    fn test_usage() {
        wakeup_screen();
    }
}
