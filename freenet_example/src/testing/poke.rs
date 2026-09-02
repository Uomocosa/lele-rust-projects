use std::process::Command;

pub fn poke() {
    let _ = Command::new("xdg-screensaver").arg("reset").output();
    let _ = Command::new("cinnamon-screensaver-command")
        .arg("--deactivate")
        .output();
}

#[cfg(test)]
mod tests {
    use super::poke;

    #[test]
    fn test_usage() {
        poke();
    }
}
