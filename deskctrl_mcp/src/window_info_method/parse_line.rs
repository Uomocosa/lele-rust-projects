use crate::WindowInfo;

/// Parse one `wmctrl -l -p -G` line.
///
/// Layout: `id desktop pid x y width height host title...` — the title is the remainder of
/// the line and may contain spaces, so only the first 8 fields are split off.
pub fn parse_line(line: &str) -> Option<WindowInfo> {
    let mut rest = line.trim_start();
    let mut fields = Vec::with_capacity(8);
    for _ in 0..8 {
        let end = rest.find(char::is_whitespace)?;
        fields.push(&rest[..end]);
        rest = rest[end..].trim_start();
    }

    Some(WindowInfo {
        id: fields[0].to_string(),
        desktop: fields[1].parse().ok()?,
        pid: fields[2].parse().ok()?,
        x: fields[3].parse().ok()?,
        y: fields[4].parse().ok()?,
        width: fields[5].parse().ok()?,
        height: fields[6].parse().ok()?,
        host: fields[7].to_string(),
        title: rest.trim_end().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_line;

    #[test]
    fn test_usage() {
        let window =
            parse_line("0x03a00004  0 2215   720  240  1200 800  uomocosa-desktop Claude").unwrap();
        assert_eq!(window.id, "0x03a00004");
        assert_eq!(window.pid, 2215);
        assert_eq!(window.width, 1200);
        assert_eq!(window.title, "Claude");

        let spaced =
            parse_line("0x04200007 -1 3312  0 0 1920 1040 host  My File - Text Editor  ").unwrap();
        assert_eq!(spaced.desktop, -1);
        assert_eq!(spaced.title, "My File - Text Editor");

        assert!(parse_line("").is_none());
        assert!(parse_line("0x1 0 nope 0 0 10 10 host title").is_none());
    }
}
