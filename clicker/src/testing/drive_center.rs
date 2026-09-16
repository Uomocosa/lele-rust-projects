use super::mouse_click_at;
use super::window_id;
use super::window_size;

/// # Errors
/// Returns an error if the window cannot be found or driven.
pub fn drive_center(title: &str, clicks: u32) -> Result<(), String> {
    let wid = window_id::window_id(title)?;
    let (width, height) = window_size::window_size(&wid)?;
    let x = width / 2;
    let y = height / 2;
    for _ in 0..clicks {
        mouse_click_at::mouse_click_at(&wid, x, y)?;
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::drive_center;

    #[test]
    fn test_usage() {
        assert!(drive_center("no-such-clicker-window-xyz", 1).is_err());
    }
}
