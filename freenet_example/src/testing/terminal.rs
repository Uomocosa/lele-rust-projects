use std::path::{Path, PathBuf};
use std::process::{Child, Command};

pub struct TerminalGuard {
    pub child: Child,
    pub window_title: String,
    pub log: PathBuf,
}

/// # Errors
/// Returns an error if `xterm`, `xdotool` or `wmctrl` is missing.
pub fn require_xterm() -> Result<(), String> {
    let out = Command::new("xterm")
        .arg("-version")
        .output()
        .map_err(|e| format!("xterm not found: {e}"))?;
    let has_output = !out.stdout.is_empty() || !out.stderr.is_empty() || out.status.success();
    if !has_output {
        return Err(
            "xterm not found on PATH — install via nix pkgs.xorg.xterm or `sudo apt install xterm`"
                .to_string(),
        );
    }
    for bin in ["xdotool", "wmctrl"] {
        let ok = Command::new("which")
            .arg(bin)
            .output()
            .is_ok_and(|o| o.status.success());
        if !ok {
            return Err(format!(
                "{bin} not found on PATH — install via nix (xdotool, wmctrl) or apt"
            ));
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error if the log cannot be created or `xterm` fails to spawn.
pub fn spawn_xterm(
    bin: &Path,
    contract_params: &str,
    tag: u64,
    log: &Path,
    title: &str,
) -> Result<TerminalGuard, String> {
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
    Ok(TerminalGuard {
        child,
        window_title: title.to_string(),
        log: log.to_path_buf(),
    })
}

/// # Errors
/// Returns an error if any `xterm` window cannot be found or tiled.
pub fn tile_three(titles: [&str; 3]) -> Result<(), String> {
    let targets = [
        (titles[0], 960, 540, 0, 0),
        (titles[1], 960, 540, 0, 540),
        (titles[2], 960, 1080, 960, 0),
    ];
    for (title, w, h, x, y) in targets {
        let mut found = false;
        for _ in 0..20 {
            if let Some(id) = find_window_id(title) {
                let _ = Command::new("xdotool")
                    .args(["windowsize", &id, &w.to_string(), &h.to_string()])
                    .output();
                let _ = Command::new("xdotool")
                    .args(["windowmove", &id, &x.to_string(), &y.to_string()])
                    .output();
                let _ = Command::new("xdotool")
                    .args(["windowactivate", &id])
                    .output();
                let _ = Command::new("wmctrl")
                    .args(["-r", title, "-e", &format!("0,{x},{y},{w},{h}")])
                    .output();
                found = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        if !found {
            return Err(format!(
                "xterm window not found for title {title:?} — FAIL fast"
            ));
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(400));
    Ok(())
}

fn find_window_id(title: &str) -> Option<String> {
    if let Ok(out) = Command::new("xdotool")
        .args(["search", "--onlyvisible", "--name", title])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        if let Some(first) = text.lines().next().map(str::trim).filter(|s| !s.is_empty()) {
            return Some(first.to_string());
        }
    }
    let out = Command::new("wmctrl").args(["-l"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.contains(title) {
            let id = line.split_whitespace().next()?;
            return Some(id.to_string());
        }
    }
    None
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
    use super::{shell_escape, tile_three};

    #[test]
    fn test_usage() {
        assert_eq!(shell_escape("abc"), "'abc'");
        assert_eq!(shell_escape("a'b"), "'a'\\''b'");
        let _ = tile_three;
    }
}
