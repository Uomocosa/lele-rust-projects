use std::path::Path;
use std::process::{Child, Command};

const DISPLAY: &str = ":0.0";

#[must_use]
pub fn start_record(clip_secs: u64, path: &Path) -> Option<Child> {
    crate::testing::wakeup_screen::wakeup_screen();
    std::thread::sleep(std::time::Duration::from_secs(5));
    if let Some(warning) = screensaver_warning() {
        eprintln!("{warning}");
    }
    let size = display_size().unwrap_or_else(|| "1920x1080".to_string());
    let path_str = path.to_string_lossy().to_string();
    let ffmpeg = ffmpeg_bin();
    let child = Command::new(ffmpeg)
        .args([
            "-y",
            "-f",
            "x11grab",
            "-framerate",
            "15",
            "-video_size",
            &size,
            "-i",
            DISPLAY,
            "-t",
            &clip_secs.to_string(),
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "30",
            "-pix_fmt",
            "yuv420p",
            &path_str,
        ])
        .spawn()
        .ok()?;
    std::thread::spawn(move || {
        for _ in 0..clip_secs.div_ceil(30) {
            std::thread::sleep(std::time::Duration::from_secs(30));
            crate::testing::wakeup_screen::poke();
        }
    });
    Some(child)
}

fn screensaver_warning() -> Option<String> {
    let out = Command::new("cinnamon-screensaver-command")
        .arg("--query")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    if text.contains("is active") {
        return Some(
            "warning: screensaver still active after wake — recording may show the blanker"
                .to_string(),
        );
    }
    None
}

fn ffmpeg_bin() -> String {
    for candidate in ["/usr/bin/ffmpeg", "ffmpeg"] {
        if has_x11grab(candidate) {
            return candidate.to_string();
        }
    }
    "/usr/bin/ffmpeg".to_string()
}

fn has_x11grab(bin: &str) -> bool {
    Command::new(bin)
        .args(["-hide_banner", "-formats"])
        .output()
        .is_ok_and(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            let err = String::from_utf8_lossy(&o.stderr);
            out.contains("x11grab") || err.contains("x11grab")
        })
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
