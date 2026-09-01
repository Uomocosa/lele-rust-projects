use std::path::Path;
use std::process::{Command, Stdio};

use crate::Error;

const FRAMERATE: &str = "15";
const VIDEO_SIZE: &str = "1920x1080";

pub fn start_record(seconds: u64, out: &Path) -> Result<std::process::Child, Error> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .args(["-f", "x11grab"])
        .args(["-r", FRAMERATE])
        .args(["-video_size", VIDEO_SIZE])
        .arg("-i")
        .arg(display())
        .args(["-t", &seconds.to_string()])
        .args(["-pix_fmt", "yuv420p"])
        .arg(out)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd.spawn()
        .map_err(|e| Error::Ffmpeg(format!("spawning ffmpeg: {e}")))
}

fn display() -> String {
    match std::env::var("DISPLAY") {
        Ok(d) if d.starts_with(':') => d,
        _ => ":0".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{FRAMERATE, VIDEO_SIZE};

    #[test]
    fn test_usage() {
        assert_eq!(FRAMERATE, "15");
        assert_eq!(VIDEO_SIZE, "1920x1080");
    }
}
