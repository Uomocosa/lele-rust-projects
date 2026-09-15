use std::path::{Path, PathBuf};
use std::process::Command;

#[must_use]
pub fn speed_clip(input: &Path, output: &Path) -> Option<PathBuf> {
    let input_str = input.to_string_lossy().to_string();
    let output_str = output.to_string_lossy().to_string();
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &input_str,
            "-filter:v",
            "setpts=0.5*PTS",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "30",
            "-pix_fmt",
            "yuv420p",
            &output_str,
        ])
        .status()
        .ok()?;
    if status.success() && output.exists() {
        Some(output.to_path_buf())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::speed_clip;

    #[test]
    fn test_usage() {
        let _ = speed_clip;
    }
}
