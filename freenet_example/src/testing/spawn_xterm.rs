use std::path::Path;
use std::process::Command;

use crate::testing;

/// # Errors
/// Returns an error if the log cannot be created or `xterm` fails to spawn.
pub fn spawn_xterm(
    bin: &Path,
    contract_params: &str,
    tag: u64,
    log: &Path,
    title: &str,
) -> Result<testing::TerminalGuard, String> {
    std::fs::File::create(log).map_err(|e| format!("create log {}: {e}", log.display()))?;
    let bin_str = bin.to_string_lossy().to_string();
    let log_str = log.to_string_lossy().to_string();
    let inner = format!(
        "stdbuf -oL -eL {} --standalone --mainnet-client --contract-params {} --instance-tag {} 2>&1 | tee -a {}; echo \"[freenet-3 #{} exited $?]\"; exec bash",
        shell_escape(&bin_str),
        shell_escape(contract_params),
        tag,
        shell_escape(&log_str),
        tag
    );
    let mut cmd = Command::new("xterm");
    cmd.args([
        "-T",
        title,
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
        child,
        window_title: title.to_string(),
        log: log.to_path_buf(),
    })
}

fn shell_escape(s: &str) -> String {
    let mut out = String::from("'");
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

#[cfg(test)]
mod tests {
    use super::{shell_escape, spawn_xterm};

    #[test]
    fn test_usage() {
        assert_eq!(shell_escape("abc"), "'abc'");
        assert_eq!(shell_escape("a'b"), "'a'\\''b'");
        let _ = spawn_xterm;
    }
}
