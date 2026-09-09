use std::path::Path;

use crate::testing;

/// # Errors
/// Returns an error if the preview cannot be read, is too small to be a real
/// capture, has no png magic, or the Telegram upload fails.
pub fn send_photo_file(
    creds: &testing::Creds,
    path: &Path,
    caption: &str,
) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|err| format!("telegram read {}: {err}", path.display()))?;
    if bytes.len() < 1024 {
        return Err(format!(
            "telegram preview too small ({} bytes): {}",
            bytes.len(),
            path.display()
        ));
    }
    if !is_png(&bytes) {
        return Err(format!("telegram preview is not png: {}", path.display()));
    }
    testing::send_photo(creds, &bytes, caption)
}

// needed helper: checks the 8-byte png signature without a decoder
fn is_png(bytes: &[u8]) -> bool {
    const MAGIC: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    bytes.len() >= MAGIC.len() && bytes.starts_with(&MAGIC)
}

#[cfg(test)]
mod tests {
    use super::send_photo_file;
    use std::path::Path;

    use crate::testing;

    #[test]
    fn test_usage() {
        let creds = testing::Creds {
            token: "t".to_string(),
            chat_id: "c".to_string(),
        };
        assert!(send_photo_file(&creds, Path::new("/nonexistent-preview.png"), "c").is_err());
        let dir = std::env::temp_dir().join("clicker-send-photo-file-test");
        let _ = std::fs::create_dir_all(&dir);
        let empty = dir.join("empty.png");
        let _ = std::fs::write(&empty, b"");
        assert!(send_photo_file(&creds, &empty, "c").is_err());
        let _ = std::fs::remove_file(&empty);
    }
}
