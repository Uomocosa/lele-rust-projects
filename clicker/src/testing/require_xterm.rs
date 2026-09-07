use std::process::Command;

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

#[cfg(test)]
mod tests {
    use super::require_xterm;

    #[test]
    fn test_usage() {
        let _ = require_xterm;
    }
}
