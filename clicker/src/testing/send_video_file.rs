use std::path::Path;

use crate::testing;

/// # Errors
/// Returns an error if the clip cannot be read, is too small to be a real
/// recording, has no mp4 `ftyp` box, or the Telegram upload fails.
pub fn send_video_file(
    creds: &testing::Creds,
    path: &Path,
    caption: &str,
) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|err| format!("telegram read {}: {err}", path.display()))?;
    if bytes.len() < 1024 {
        return Err(format!(
            "telegram clip too small ({} bytes): {}",
            bytes.len(),
            path.display()
        ));
    }
    if !bytes.windows(4).any(|w| w == b"ftyp") {
        return Err(format!(
            "telegram clip is not mp4 (no ftyp box): {}",
            path.display()
        ));
    }
    testing::send_video(creds, &bytes, caption)
}

#[cfg(test)]
mod tests {
    use super::send_video_file;
    use std::path::Path;

    use crate::testing;

    #[test]
    fn test_usage() {
        let creds = testing::Creds {
            token: "t".to_string(),
            chat_id: "c".to_string(),
        };
        assert!(send_video_file(&creds, Path::new("/nonexistent-clip.mp4"), "c").is_err());
        let dir = std::env::temp_dir().join("clicker-send-video-file-test");
        let _ = std::fs::create_dir_all(&dir);
        let empty = dir.join("empty.mp4");
        let _ = std::fs::write(&empty, b"");
        assert!(send_video_file(&creds, &empty, "c").is_err());
        let _ = std::fs::remove_file(&empty);
    }
}
