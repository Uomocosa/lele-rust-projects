use std::path::Path;

use crate::Creds;
use crate::Error;
use crate::send_video;

/// # Errors
/// Returns an error if the clip cannot be read, is too small to be a real
/// recording, has no mp4 `ftyp` box, or the Telegram upload fails.
pub fn send_video_file(creds: &Creds, path: &Path, caption: &str) -> Result<String, Error> {
    let display = path.display().to_string();
    let bytes = std::fs::read(path).map_err(|source| Error::Read {
        path: display.clone(),
        source,
    })?;
    if bytes.len() < 1024 {
        return Err(Error::TooSmall {
            kind: "clip",
            path: display,
            bytes: bytes.len(),
        });
    }
    if !bytes.windows(4).any(|window| window == b"ftyp") {
        return Err(Error::NotMp4 { path: display });
    }
    send_video::send_video(creds, &bytes, caption)
}

#[cfg(test)]
mod tests {
    use super::send_video_file;
    use std::path::Path;

    use crate::Creds;

    #[test]
    fn test_usage() {
        let creds = Creds {
            token: "token".to_string(),
            chat_id: "chat".to_string(),
        };
        assert!(send_video_file(&creds, Path::new("/nonexistent-clip.mp4"), "c").is_err());
        let dir = std::env::temp_dir().join("telegram-bot-send-video-file-test");
        let _ = std::fs::create_dir_all(&dir);
        let empty = dir.join("empty.mp4");
        let _ = std::fs::write(&empty, b"");
        assert!(send_video_file(&creds, &empty, "c").is_err());
        let _ = std::fs::remove_file(&empty);
    }
}
