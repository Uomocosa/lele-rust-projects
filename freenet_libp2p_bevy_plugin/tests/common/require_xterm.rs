use std::process::Command;

/// # Errors
/// Returns an error if `xterm`, `xdotool` or `wmctrl` is missing.
pub fn require_xterm() -> Result<(), String> {
    let out = Command::new("xterm")
        .arg("-version")
        .output()
        .map_err(|e| format!("xterm not found: {e}"))?;
    if !out.status.success() && out.stdout.is_empty() && out.stderr.is_empty() {
        return Err("xterm not found on PATH".to_string());
    }
    for bin in ["xdotool", "wmctrl"] {
        let ok = Command::new("which")
            .arg(bin)
            .output()
            .is_ok_and(|o| o.status.success());
        if !ok {
            return Err(format!("{bin} not found on PATH"));
        }
    }
    Ok(())
}
