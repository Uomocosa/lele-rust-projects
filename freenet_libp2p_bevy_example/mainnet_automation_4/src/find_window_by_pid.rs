use crate::window_info;

pub fn find_window_by_pid(
    windows: &[window_info::WindowInfo],
    pid: u32,
) -> Option<window_info::WindowInfo> {
    windows.iter().find(|w| w.pid == pid).cloned()
}

// no test_usage necessary
