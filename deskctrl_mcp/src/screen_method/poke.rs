use std::process::Command;

/// Report activity so an idle timer never fires, without the heavier session-unlock and DPMS
/// steps of `wake`. Meant to be called on a timer for as long as a capture is running.
pub fn poke() {
    let _ = Command::new("xdg-screensaver").arg("reset").output();
    let _ = Command::new("cinnamon-screensaver-command")
        .arg("--deactivate")
        .output();
}

#[cfg(test)]
mod tests {
    use super::poke;
    use crate::test_support;

    #[test]
    fn test_usage() {
        test_support::assert_live_display();
        poke();
    }
}
