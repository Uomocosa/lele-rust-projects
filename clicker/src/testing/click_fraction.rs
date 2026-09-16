use super::mouse_click_at;
use super::window_id;
use super::window_size;

/// # Errors
/// Returns an error if the window cannot be found, measured, or clicked.
pub fn click_fraction(title: &str, fx: f64, fy: f64) -> Result<(), String> {
    let wid = window_id::window_id(title)?;
    let (width, height) = window_size::window_size(&wid)?;
    let x = (f64::from(width) * fx) as i32;
    let y = f64::from(height).mul_add(fy, 0.0) as i32;
    mouse_click_at::mouse_click_at(&wid, x, y)
}

#[cfg(test)]
mod tests {
    use super::click_fraction;

    #[test]
    fn test_usage() {
        assert!(click_fraction("no-such-clicker-window-xyz", 0.5, 0.5).is_err());
    }
}
