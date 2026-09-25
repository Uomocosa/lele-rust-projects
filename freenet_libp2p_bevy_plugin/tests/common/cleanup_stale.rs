use std::process::Command;
use std::time::Duration;

pub fn cleanup_stale() {
    for pattern in ["examples/lobby_room", "xterm.*lobby-"] {
        let _ = Command::new("pkill").args(["-f", pattern]).output();
    }
    std::thread::sleep(Duration::from_secs(2));
}
