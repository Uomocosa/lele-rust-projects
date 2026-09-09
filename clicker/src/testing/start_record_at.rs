use std::path::Path;
use std::process::{Child, Command};

use crate::testing;

#[must_use]
pub fn start_record_at(clip_secs: u64, path: &Path, x: i32, y: i32) -> Option<Child> {
    testing::wakeup_screen();
    std::thread::sleep(std::time::Duration::from_secs(2));
    let path_str = path.to_string_lossy().to_string();
    let input = format!(":0.0+{x},{y}");
    let child = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "x11grab",
            "-framerate",
            "15",
            "-video_size",
            "800x450",
            "-i",
            &input,
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
            testing::poke();
        }
    });
    Some(child)
}

#[cfg(test)]
mod tests {
    use super::start_record_at;

    #[test]
    fn test_usage() {
        let _ = start_record_at;
    }
}
