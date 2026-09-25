use std::path::Path;
use std::process::Command;

use super::shell_escape::shell_escape;
use super::terminal_guard::TerminalGuard;

/// # Errors
/// Returns an error if the log cannot be created or `xterm` fails to spawn.
pub fn spawn_xterm(
    bin: &Path,
    args: &[String],
    username: &str,
    log: &Path,
) -> Result<TerminalGuard, String> {
    std::fs::File::create(log).map_err(|e| format!("create log {}: {e}", log.display()))?;
    let title = format!("lobby-{username}");
    let mut command = format!("stdbuf -oL -eL {} ", shell_escape(&bin.to_string_lossy()));
    for arg in args {
        command.push_str(&shell_escape(arg));
        command.push(' ');
    }
    let inner = format!(
        "{command}2>&1 | tee -a {}; echo \"[lobby {username} exited $?]\"; exec bash",
        shell_escape(&log.to_string_lossy())
    );
    let child = Command::new("xterm")
        .args([
            "-T",
            &title,
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
        ])
        .spawn()
        .map_err(|e| format!("spawn xterm failed: {e}"))?;
    std::thread::sleep(std::time::Duration::from_millis(600));
    Ok(TerminalGuard {
        child: Some(child),
        window_title: title,
        log: log.to_path_buf(),
    })
}
