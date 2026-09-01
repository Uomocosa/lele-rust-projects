use std::path::Path;

use crate::testing;

pub fn send_video_file(creds: &testing::Creds, path: &Path, caption: &str) {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(err) => {
            eprintln!(
                "telegram send_video_file read failed: {} err={err}",
                path.display()
            );
            return;
        }
    };
    if bytes.is_empty() {
        eprintln!("telegram send_video_file empty: {}", path.display());
        return;
    }
    testing::send_video(creds, &bytes, caption);
}

#[cfg(test)]
mod tests {
    use super::send_video_file;

    #[test]
    fn test_usage() {
        let _ = send_video_file;
    }
}
