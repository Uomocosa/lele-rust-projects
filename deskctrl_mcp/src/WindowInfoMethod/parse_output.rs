use crate::{WindowInfo, WindowInfoMethod::parse_line};

/// Parse full `wmctrl -l -p -G` output, dropping unparseable lines and the desktop-manager
/// pseudo-window (Nemo/Nautilus render the desktop as a window titled "Desktop").
pub fn parse_output(output: &str) -> Vec<WindowInfo> {
    output
        .lines()
        .filter_map(parse_line)
        .filter(|w| w.title != "Desktop")
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_output;

    #[test]
    fn test_usage() {
        let output = "0x03800003  0 2223   0    0    1920 1040 uomocosa-desktop Desktop\n\
                      0x03a00004  0 2215   720  240  1200 800  uomocosa-desktop Claude\n\
                      garbage\n";
        let windows = parse_output(output);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].title, "Claude");
    }
}
