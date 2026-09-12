use std::path::Path;

use crate::Creds;
use crate::Error;
use crate::send_photo;

/// # Errors
/// Returns an error if the preview cannot be read, is too small to be a real
/// capture, has no png magic, or the Telegram upload fails.
pub fn send_photo_file(creds: &Creds, path: &Path, caption: &str) -> Result<String, Error> {
    let display = path.display().to_string();
    let bytes = std::fs::read(path).map_err(|source| Error::Read {
        path: display.clone(),
        source,
    })?;
    if bytes.len() < 1024 {
        return Err(Error::TooSmall {
            kind: "preview",
            path: display,
            bytes: bytes.len(),
        });
    }
    if !is_png(&bytes) {
        return Err(Error::NotPng { path: display });
    }
    send_photo::send_photo(creds, &bytes, caption)
}

fn is_png(bytes: &[u8]) -> bool {
    const MAGIC: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    bytes.len() >= MAGIC.len() && bytes.starts_with(&MAGIC)
}

#[cfg(test)]
mod tests {
    use super::send_photo_file;
    use std::path::Path;

    use crate::Creds;

    #[test]
    fn test_usage() {
        let creds = Creds {
            token: "token".to_string(),
            chat_id: "chat".to_string(),
        };
        assert!(send_photo_file(&creds, Path::new("/nonexistent-preview.png"), "c").is_err());
        let dir = std::env::temp_dir().join("telegram-bot-send-photo-file-test");
        let _ = std::fs::create_dir_all(&dir);
        let empty = dir.join("empty.png");
        let _ = std::fs::write(&empty, b"");
        assert!(send_photo_file(&creds, &empty, "c").is_err());
        let _ = std::fs::remove_file(&empty);
    }
}
