use std::process::Command;
use std::time::Duration;

use x11rb::protocol::xproto::{ConnectionExt as _, InputFocus};
use x11rb::rust_connection::RustConnection;

use crate::Error;

const FOCUS_SETTLE_MS: u64 = 400;

pub fn raise_window(window_id: &str) -> Result<(), Error> {
    Command::new("wmctrl")
        .args(["-i", "-a", window_id])
        .status()
        .ok();
    std::thread::sleep(Duration::from_millis(FOCUS_SETTLE_MS));

    let window = parse_window_id(window_id)?;
    let (conn, _) = connect()?;
    if input_focus_is(&conn, window)? {
        return Ok(());
    }
    set_input_focus(&conn, window)?;
    std::thread::sleep(Duration::from_millis(FOCUS_SETTLE_MS));
    if input_focus_is(&conn, window)? {
        return Ok(());
    }
    Err(Error::Window(format!(
        "could not focus window {window_id}: wmctrl -a and SetInputFocus both failed"
    )))
}

fn connect() -> Result<(RustConnection, usize), Error> {
    x11rb::connect(None).map_err(|e| Error::X11(format!("connecting to X display: {e}")))
}

// needed helper:
fn parse_window_id(window_id: &str) -> Result<u32, Error> {
    u32::from_str_radix(window_id.trim_start_matches("0x"), 16)
        .map_err(|e| Error::Window(format!("parsing window_id {window_id:?}: {e}")))
}

// needed helper:
fn input_focus_is(conn: &RustConnection, window: u32) -> Result<bool, Error> {
    let focus = conn
        .get_input_focus()
        .map_err(|e| Error::X11(format!("get_input_focus: {e}")))?
        .reply()
        .map_err(|e| Error::X11(format!("get_input_focus reply: {e}")))?;
    Ok(focus.focus == window)
}

fn set_input_focus(conn: &RustConnection, window: u32) -> Result<(), Error> {
    conn.set_input_focus(InputFocus::PARENT, window, 0u32)
        .map_err(|e| Error::X11(format!("set_input_focus: {e}")))?
        .check()
        .map_err(|e| Error::X11(format!("set_input_focus failed: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_window_id;

    #[test]
    fn test_usage() {
        assert_eq!(parse_window_id("0x0560000e").unwrap(), 0x0560000e);
        assert!(parse_window_id("nope").is_err());
    }
}
