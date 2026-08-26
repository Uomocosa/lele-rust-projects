use std::process::Command;

use crate::Error;
use crate::window_info;

pub fn list_windows() -> Result<Vec<window_info::WindowInfo>, Error> {
    let output = Command::new("wmctrl")
        .args(["-l", "-p", "-G"])
        .output()
        .map_err(|e| Error::Window(format!("spawning wmctrl: {e}")))?;
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.lines().filter_map(parse_line).collect())
}

// needed helper:
fn parse_line(line: &str) -> Option<window_info::WindowInfo> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut parts = trimmed.split_whitespace();
    let id = parts.next()?.to_string();
    parts.next()?;
    let pid = parts.next()?.parse::<u32>().ok()?;
    let x = parts.next()?.parse::<i32>().ok()?;
    let y = parts.next()?.parse::<i32>().ok()?;
    let width = parts.next()?.parse::<u32>().ok()?;
    let height = parts.next()?.parse::<u32>().ok()?;
    parts.next()?;
    let title = parts.collect::<Vec<_>>().join(" ");
    Some(window_info::WindowInfo {
        id,
        pid,
        x,
        y,
        width,
        height,
        title,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_line;

    #[test]
    fn test_usage() {
        let line = "0x03a00004   0 2215   720   240 1200 800 uomocosa-desktop xterm";
        let w = parse_line(line).unwrap();
        assert_eq!(w.id, "0x03a00004");
        assert_eq!(w.pid, 2215);
        assert_eq!(w.width, 1200);
        assert!(parse_line("   ").is_none());
    }
}
