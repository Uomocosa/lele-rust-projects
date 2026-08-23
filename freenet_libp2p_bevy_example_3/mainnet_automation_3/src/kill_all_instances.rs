use std::process::Command;

use crate::Error;

const BIN_MATCH: &str = "freenet-libp2p-bevy-example-3";

pub fn kill_all_instances() -> Result<(), Error> {
    let killed = Command::new("pkill")
        .arg("-f")
        .arg(BIN_MATCH)
        .status()
        .map_err(|e| Error::Teardown(format!("spawning pkill: {e}")))?;
    if killed.success() {
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    let remaining = Command::new("pgrep")
        .arg("-af")
        .arg(BIN_MATCH)
        .output()
        .map_err(|e| Error::Teardown(format!("spawning pgrep: {e}")))?;
    let text = String::from_utf8_lossy(&remaining.stdout);
    if text.trim().is_empty() {
        Ok(())
    } else {
        Err(Error::Teardown(format!(
            "instances still running after kill: {text}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::BIN_MATCH;

    #[test]
    fn test_usage() {
        assert!(BIN_MATCH.contains("example-3"));
    }
}
