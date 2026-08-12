use std::process::Command;
use std::time::Duration;

use x11rb::protocol::xproto::{ConnectionExt as _, InputFocus};
use x11rb::rust_connection::RustConnection;

use crate::Error;

const FOCUS_SETTLE_MS: u64 = 400;

/// Raise and activate `window_id`, verifying that the X input focus actually lands on the window
/// before returning. First asks the window manager (`wmctrl -i -a`); if that does not give the
/// window focus, falls back to `SetInputFocus` directly, which works even without a cooperating
/// WM. Returns an error when neither path focuses the window.
///
/// Note this verifies only the *logical* focus: an active keyboard grab by another client (e.g.
/// a modal dialog) keeps XTEST keystrokes flowing to the grabber no matter where focus points.
/// Callers that send keys must additionally run `assert_no_keyboard_grab` before raising.
pub fn raise_window(window_id: &str) -> Result<(), Error> {
    Command::new("wmctrl")
        .args(["-i", "-a", window_id])
        .status()
        .ok();
    std::thread::sleep(Duration::from_millis(FOCUS_SETTLE_MS));

    let window = parse_window_id(window_id)?;
    let (conn, _screen) = connect()?;
    if input_focus_is(&conn, window)? {
        return Ok(());
    }

    set_input_focus(&conn, window)?;
    std::thread::sleep(Duration::from_millis(FOCUS_SETTLE_MS));

    if input_focus_is(&conn, window)? {
        return Ok(());
    }

    Err(Error::Window(format!(
        "could not move keyboard focus to window {window_id}: wmctrl -a and SetInputFocus both \
         failed, so keystrokes would land on the currently focused window instead"
    )))
}

// needed helper:
fn connect() -> Result<(RustConnection, usize), Error> {
    x11rb::connect(None).map_err(|e| Error::Window(format!("connecting to X display: {e}")))
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
        .map_err(|e| Error::Window(format!("get_input_focus request: {e}")))?
        .reply()
        .map_err(|e| Error::Window(format!("get_input_focus reply: {e}")))?;
    Ok(focus.focus == window)
}

// needed helper:
fn set_input_focus(conn: &RustConnection, window: u32) -> Result<(), Error> {
    conn.set_input_focus(InputFocus::PARENT, window, 0u32 /* CurrentTime */)
        .map_err(|e| Error::Window(format!("set_input_focus request: {e}")))?
        .check()
        .map_err(|e| Error::Window(format!("set_input_focus failed: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_window_id;
    use crate::{test_support, window_info_method};

    #[test]
    fn test_usage() {
        assert_eq!(parse_window_id("0x0560000e").unwrap(), 0x0560000e);
        assert_eq!(parse_window_id("0x3c00004").unwrap(), 0x3c00004);
        assert!(parse_window_id("nope").is_err());
    }

    /// Spawns a real xterm and verifies `raise_window` actually moves X input focus onto it.
    #[tokio::test]
    #[ignore]
    async fn test_usage_live_focus() {
        test_support::assert_live_display();
        let _guard = test_support::live_test_lock().lock().await;

        let mut child = std::process::Command::new("xterm")
            .spawn()
            .expect("spawning xterm for live raise_window test");
        std::thread::sleep(std::time::Duration::from_millis(800));

        let windows = window_info_method::list().expect("list_windows for live raise_window test");
        let window = windows
            .iter()
            .find(|w| w.pid == child.id())
            .expect("xterm window not found by pid");

        super::raise_window(&window.id).expect("raise_window should focus the xterm");

        let _ = child.kill();
        let _ = child.wait();
    }
}
