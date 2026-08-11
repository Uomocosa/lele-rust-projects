use std::process::Command;

pub fn raise_window(window_id: &str) {
    let _ = Command::new("wmctrl")
        .args(["-i", "-a", window_id])
        .output();
    std::thread::sleep(std::time::Duration::from_millis(400));
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        // real coverage is the live X11 tests in click_window and send_keys
    }
}
