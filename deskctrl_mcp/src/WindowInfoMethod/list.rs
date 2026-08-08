use std::process::Command;

use crate::{Error, WindowInfo, WindowInfoMethod::parse_output};

/// All open desktop windows, via `wmctrl -l -p -G`.
pub fn list() -> Result<Vec<WindowInfo>, Error> {
    let out = Command::new("wmctrl")
        .args(["-l", "-p", "-G"])
        .output()
        .map_err(|e| Error::Window(format!("running wmctrl: {e} (is wmctrl installed?)")))?;

    if !out.status.success() {
        return Err(Error::Window(format!(
            "wmctrl failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }

    Ok(parse_output(&String::from_utf8_lossy(&out.stdout)))
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires a live X display"]
    fn test_usage_live_display() {
        assert!(super::list().is_ok());
    }
}
// no test_usage necessary: needs a live X display; its parsing lives in parse_output.rs
