/// A single open desktop window, as reported by `wmctrl -l -p -G`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    /// X11 window id, e.g. "0x03a00004".
    pub id: String,
    /// Virtual desktop number (-1 for sticky windows).
    pub desktop: i32,
    /// Owning process id. 0 when the window exposes no _NET_WM_PID.
    pub pid: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub host: String,
    pub title: String,
}

#[rustfmt::skip]
impl WindowInfo {
    pub fn geometry(&self) -> String { crate::window_info_method::geometry(self) }
}

#[cfg(test)]
mod tests {
    use crate::WindowInfo;

    #[test]
    fn test_usage() {
        let window = WindowInfo {
            id: "0x03a00004".to_string(),
            desktop: 0,
            pid: 2215,
            x: 720,
            y: 240,
            width: 1200,
            height: 800,
            host: "uomocosa-desktop".to_string(),
            title: "Claude".to_string(),
        };
        assert_eq!(window.geometry(), "1200x800+720+240");
    }
}
