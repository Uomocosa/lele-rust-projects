use rmcp::model::{CallToolResult, ContentBlock};
use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{ConnectionExt as _, MOTION_NOTIFY_EVENT},
        xtest::ConnectionExt as _,
    },
};

use crate::{ClickParams, Error, window_info_method};

const BUTTON_PRESS_EVENT: u8 = 4;
const BUTTON_RELEASE_EVENT: u8 = 5;

pub async fn click_window(params: ClickParams) -> Result<CallToolResult, Error> {
    let ClickParams {
        window_id,
        x,
        y,
        button,
        ..
    } = params;

    if !window_info_method::is_valid_id(&window_id) {
        return Err(Error::Window(format!(
            "invalid window_id {window_id:?}: expected hex like \"0x03a00004\" (see list_windows)"
        )));
    }
    if !(1..=5).contains(&button) {
        return Err(Error::Window(format!(
            "invalid button {button}: expected 1 (left), 2 (middle) or 3 (right)"
        )));
    }
    let window = u32::from_str_radix(window_id.trim_start_matches("0x"), 16)
        .map_err(|e| Error::Window(format!("parsing window_id {window_id:?}: {e}")))?;

    // XTEST synthesizes input at the *root*, so it lands on whatever is topmost at those
    // coordinates. Raise the target first or an overlapping window eats the click.
    super::raise_window::raise_window(&window_id);

    let (conn, _screen) =
        x11rb::connect(None).map_err(|e| Error::Window(format!("connecting to X display: {e}")))?;

    // Ask X where the window actually is rather than trusting wmctrl's geometry, which does not
    // always agree with the absolute offset under scaling.
    let root = conn.setup().roots[0].root;
    let point = conn
        .translate_coordinates(window, root, x, y)
        .map_err(|e| Error::Window(format!("translate_coordinates request: {e}")))?
        .reply()
        .map_err(|e| Error::Window(format!("no such window {window_id}: {e}")))?;

    conn.xtest_fake_input(MOTION_NOTIFY_EVENT, 0, 0, root, point.dst_x, point.dst_y, 0)
        .and_then(|_| conn.xtest_fake_input(BUTTON_PRESS_EVENT, button, 0, root, 0, 0, 0))
        .and_then(|_| conn.xtest_fake_input(BUTTON_RELEASE_EVENT, button, 0, root, 0, 0, 0))
        .map_err(|e| Error::Window(format!("sending click: {e}")))?;
    conn.flush()
        .map_err(|e| Error::Window(format!("flushing click: {e}")))?;

    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "clicked button {button} at ({x}, {y}) in {window_id} \
         = root ({}, {}); screenshot the window to confirm it took effect",
        point.dst_x, point.dst_y
    ))]))
}

#[cfg(test)]
mod tests {
    use super::click_window;
    use crate::test_support;
    use crate::{ClickParams, Error};

    #[tokio::test]
    async fn test_usage() {
        // Rejected before any X connection or wmctrl call is attempted.
        let bad_id = click_window(ClickParams {
            window_id: "nope".to_string(),
            x: 0,
            y: 0,
            button: 1,
            note: None,
            send_to_telegram: true,
        })
        .await;
        assert!(matches!(bad_id, Err(Error::Window(_))));

        let bad_button = click_window(ClickParams {
            window_id: "0x1".to_string(),
            x: 0,
            y: 0,
            button: 9,
            note: None,
            send_to_telegram: true,
        })
        .await;
        assert!(matches!(bad_button, Err(Error::Window(_))));
    }

    /// Spawns a real xterm, finds its window, and clicks inside it. No pixel-diffing (too
    /// flaky) — this only confirms the full X connect / translate_coordinates / xtest_fake_input
    /// path doesn't error against a real window.
    #[tokio::test]
    async fn test_usage_live_display() {
        use crate::window_info_method;

        test_support::assert_live_display();
        let _guard = test_support::live_test_lock().lock().await;

        let mut child = std::process::Command::new("xterm")
            .spawn()
            .expect("spawning xterm for live click test");
        std::thread::sleep(std::time::Duration::from_millis(800));

        let windows = window_info_method::list().expect("list_windows for live click test");
        let window = windows
            .iter()
            .find(|w| w.pid == child.id())
            .expect("xterm window not found by pid");

        let result = click_window(ClickParams {
            window_id: window.id.clone(),
            x: 10,
            y: 10,
            button: 1,
            note: None,
            send_to_telegram: true,
        })
        .await;

        let _ = child.kill();
        let _ = child.wait();

        assert!(result.is_ok(), "{result:?}");
    }
}
