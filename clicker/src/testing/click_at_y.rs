use super::mouse_click_at;
use super::window_id;
use super::window_size;

/// # Errors
/// Returns an error if the window cannot be found, measured, or clicked.
pub fn click_at_y(title: &str, fx: f64, y: i32) -> Result<(), String> {
    let wid = window_id::window_id(title)?;
    let (width, _) = window_size::window_size(&wid)?;
    let x = f64::from(width) * fx;
    mouse_click_at::mouse_click_at(&wid, x, f64::from(y))
}

#[cfg(test)]
mod tests {
    use super::click_at_y;

    #[test]
    fn test_usage() {
        assert!(click_at_y("no-such-clicker-window-xyz", 0.5, 80).is_err());
    }
}
