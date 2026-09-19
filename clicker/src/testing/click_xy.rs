use super::mouse_click_at;
use super::window_id;

/// # Errors
/// Returns an error if the window cannot be found or the click cannot be sent.
pub fn click_xy(title: &str, x: f64, y: f64) -> Result<(), String> {
    let wid = window_id::window_id(title)?;
    mouse_click_at::mouse_click_at(&wid, x, y)
}

// no test_usage necessary
