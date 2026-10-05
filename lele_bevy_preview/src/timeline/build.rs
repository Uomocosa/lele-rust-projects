use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Error;

const DEFAULT_FFMPEG: &str = "ffmpeg";

pub fn build(frames: &[PathBuf], fps: u32, out: &Path, scratch: &Path) -> Result<PathBuf, Error> {
    if frames.is_empty() {
        return Err(Error::Ffmpeg {
            path: out.display().to_string(),
            message: String::from("no frames were captured"),
        });
    }
    std::fs::create_dir_all(scratch).map_err(|error| Error::Io(error.to_string()))?;
    let destination = out.to_path_buf();
    let status = Command::new(ffmpeg_binary())
        .arg("-y")
        .arg("-framerate")
        .arg(fps.to_string())
        .arg("-i")
        .arg(scratch.join("frame-%04d.png"))
        .arg("-c:v")
        .arg("libx264")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-vf")
        .arg("pad=ceil(iw/2)*2:ceil(ih/2)*2")
        .arg("-r")
        .arg(fps.to_string())
        .arg(&destination)
        .status()
        .map_err(|error| Error::Ffmpeg {
            path: destination.display().to_string(),
            message: error.to_string(),
        })?;
    if !status.success() {
        return Err(Error::Ffmpeg {
            path: destination.display().to_string(),
            message: format!("ffmpeg exited with {status}"),
        });
    }
    Ok(destination)
}

// needed helper: nix store paths are not on PATH, so LELE_FFMPEG overrides the binary name
fn ffmpeg_binary() -> String {
    match std::env::var("LELE_FFMPEG") {
        Ok(path) if !path.trim().is_empty() => path,
        _ => String::from(DEFAULT_FFMPEG),
    }
}
// no test_usage necessary
