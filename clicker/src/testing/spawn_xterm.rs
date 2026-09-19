use std::process::Command;

use super::xterm_spec::XtermSpec;
use crate::testing;

/// # Errors
/// Returns an error if the log cannot be created or `xterm` fails to spawn.
pub fn spawn_xterm(spec: &XtermSpec) -> Result<testing::TerminalGuard, String> {
    std::fs::File::create(spec.log)
        .map_err(|e| format!("create log {}: {e}", spec.log.display()))?;
    let title = format!("clicker-xterm-{}", spec.tag);
    let inner = spec.command();
    let mut cmd = Command::new("xterm");
    cmd.args([
        "-T",
        title.as_str(),
        "-fa",
        "Monospace",
        "-fs",
        "10",
        "-bg",
        "black",
        "-fg",
        "white",
        "-e",
        "bash",
        "-lc",
        &inner,
    ]);
    let child = cmd
        .spawn()
        .map_err(|e| format!("spawn xterm failed: {e}"))?;
    std::thread::sleep(std::time::Duration::from_millis(600));
    Ok(testing::TerminalGuard {
        child: Some(child),
        window_title: title,
        log: spec.log.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::spawn_xterm;

    #[test]
    fn test_usage() {
        let _ = spawn_xterm;
    }
}
