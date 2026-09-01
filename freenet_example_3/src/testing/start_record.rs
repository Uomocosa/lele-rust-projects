use std::path::Path;
use std::process::{Child, Command};

const DISPLAY: &str = ":0.0";

#[must_use]
pub fn start_record(clip_secs: u64, path: &Path) -> Option<Child> {
    let size = display_size().unwrap_or_else(|| "1920x1080".to_string());
    let path_str = path.to_string_lossy().to_string();
    Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "x11grab",
            "-video_size",
            &size,
            "-i",
            DISPLAY,
            "-t",
            &clip_secs.to_string(),
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-preset",
            "ultrafast",
            &path_str,
        ])
        .spawn()
        .ok()
}

fn display_size() -> Option<String> {
    let out = Command::new("xdpyinfo").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.contains("dimensions:") {
            let part = line.split("dimensions:").nth(1)?;
            let dims = part.split_whitespace().next()?;
            return Some(dims.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::display_size;

    #[test]
    fn test_usage() {
        let _ = display_size();
    }
}
