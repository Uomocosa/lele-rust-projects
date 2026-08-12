use x11rb::{
    connection::Connection,
    protocol::xproto::{ConnectionExt as _, GrabMode, GrabStatus},
    rust_connection::RustConnection,
};

use crate::Error;

/// Detect an active keyboard grab by another client (e.g. a modal dialog) and error out. XTEST
/// routes fake input to the grabbing client, so keystrokes would land in that window regardless
/// of where focus points. Probes by attempting `grab_keyboard` on the root: an active grab by
/// someone else returns `ALREADY_GRABBED`; when this call succeeds it grabs the keyboard
/// ourselves, so the grab is released immediately (with a round-trip) before the caller sends
/// any keystrokes.
pub fn assert_no_keyboard_grab() -> Result<(), Error> {
    let (conn, _screen) = connect()?;
    let root = conn.setup().roots[0].root;
    let reply = conn
        .grab_keyboard(false, root, 0u32, GrabMode::ASYNC, GrabMode::ASYNC)
        .map_err(|e| Error::Window(format!("grab_keyboard request: {e}")))?
        .reply()
        .map_err(|e| Error::Window(format!("grab_keyboard reply: {e}")))?;
    match reply.status {
        GrabStatus::SUCCESS => {
            // `.check()` forces a round-trip so the server has processed the release before
            // keystrokes are sent on a separate connection (they would otherwise route to us).
            conn.ungrab_keyboard(0u32)
                .map_err(|e| Error::Window(format!("ungrab_keyboard request: {e}")))?
                .check()
                .map_err(|e| Error::Window(format!("ungrab_keyboard failed: {e}")))?;
            Ok(())
        }
        GrabStatus::ALREADY_GRABBED => Err(Error::Window(
            "keyboard is actively grabbed by another window (e.g. a modal dialog): keystrokes \
             would go there instead of the target window, dismiss it and retry"
                .to_string(),
        )),
        GrabStatus::FROZEN => Err(Error::Window(
            "keyboard is frozen by an active grab of another client".to_string(),
        )),
        other => Err(Error::Window(format!(
            "grab_keyboard returned {other:?}, so keystrokes may not reach the target window"
        ))),
    }
}

// needed helper:
fn connect() -> Result<(RustConnection, usize), Error> {
    x11rb::connect(None).map_err(|e| Error::Window(format!("connecting to X display: {e}")))
}

#[cfg(test)]
mod tests {
    use x11rb::{
        connection::Connection,
        protocol::xproto::{ConnectionExt as _, GrabMode, GrabStatus},
    };

    use super::assert_no_keyboard_grab;
    use crate::test_support;

    /// Grabs the keyboard on its own X connection, asserts the check errors while the grab is
    /// held, then releases it and asserts the check passes again. When another client already
    /// holds an active grab (e.g. a lingering modal dialog), only the error path is verifiable.
    #[tokio::test]
    #[ignore]
    async fn test_usage() {
        test_support::assert_live_display();
        let _guard = test_support::live_test_lock().lock().await;

        let (conn, _screen) = x11rb::connect(None).expect("connecting to X display for grab test");
        let root = conn.setup().roots[0].root;
        let grabbed = conn
            .grab_keyboard(false, root, 0u32, GrabMode::ASYNC, GrabMode::ASYNC)
            .expect("grab_keyboard request")
            .reply()
            .expect("grab_keyboard reply");
        if grabbed.status == GrabStatus::ALREADY_GRABBED {
            assert!(
                assert_no_keyboard_grab().is_err(),
                "an active keyboard grab must be detected"
            );
            return;
        }
        assert_eq!(grabbed.status, GrabStatus::SUCCESS);

        assert!(
            assert_no_keyboard_grab().is_err(),
            "an active keyboard grab must be detected"
        );

        conn.ungrab_keyboard(0u32)
            .expect("ungrab_keyboard request")
            .check()
            .expect("ungrab_keyboard reply");
        std::thread::sleep(std::time::Duration::from_millis(100));

        assert!(
            assert_no_keyboard_grab().is_ok(),
            "a released grab must pass the check"
        );
    }
}
