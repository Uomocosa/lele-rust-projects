use std::process::Command;

use crate::{Error, WindowInfo, window_info_method::parse_output};

/// All open desktop windows, via `wmctrl -l -p -G`.
///
/// A window closing between wmctrl's enumeration and its property query races the X server and
/// makes wmctrl exit nonzero (BadWindow); retry once after a brief settle rather than surfacing
/// a spurious failure for a window the caller never cared about.
pub fn list() -> Result<Vec<WindowInfo>, Error> {
    match run() {
        Ok(windows) => Ok(windows),
        Err(_) => {
            std::thread::sleep(std::time::Duration::from_millis(150));
            run()
        }
    }
}

fn run() -> Result<Vec<WindowInfo>, Error> {
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
    use crate::test_support;
    #[test]
    fn test_usage_live_display() {
        test_support::assert_live_display();
        let _guard = test_support::live_test_lock().blocking_lock();

        assert!(super::list().is_ok());
    }
}
// no test_usage necessary: needs a live X display; its parsing lives in parse_output.rs
