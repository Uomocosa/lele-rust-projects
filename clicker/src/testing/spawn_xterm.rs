use std::path::Path;
use std::process::Command;

use crate::testing;

/// # Errors
/// Returns an error if the log cannot be created or `xterm` fails to spawn.
pub fn spawn_xterm(
    bin: &Path,
    namespace: &str,
    lobby: Option<&str>,
    create: bool,
    tag: u64,
    contract_params: &str,
    since_epoch: Option<u64>,
    transport: &str,
    mdns: bool,
    log: &Path,
) -> Result<testing::TerminalGuard, String> {
    std::fs::File::create(log).map_err(|e| format!("create log {}: {e}", log.display()))?;
    let title = format!("clicker-xterm-{tag}");
    let bin_str = bin.to_string_lossy().to_string();
    let log_str = log.to_string_lossy().to_string();
    let create_arg = if create { " --create-lobby" } else { "" };
    let lobby_arg = lobby.map_or(String::new(), |room| {
        format!(" --lobby {}", shell_escape(room))
    });
    let since_arg = since_epoch.map_or(String::new(), |epoch| format!(" --since-epoch {epoch}"));
    let mdns_arg = if mdns {
        String::new()
    } else {
        " --disable-mdns".to_string()
    };
    let inner = format!(
        "stdbuf -oL -eL {} --namespace {} {}{} --instance-tag {} --own-id {} --contract-params {}{} --transport {}{} 2>&1 | tee -a {}; echo \"[clicker-3 #{} exited $?]\"; exec bash",
        shell_escape(&bin_str),
        shell_escape(namespace),
        lobby_arg,
        create_arg,
        tag,
        tag,
        shell_escape(contract_params),
        since_arg,
        shell_escape(transport),
        mdns_arg,
        shell_escape(&log_str),
        tag
    );
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
