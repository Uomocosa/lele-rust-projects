use std::path::Path;

use super::load_creds;
use super::send_video_file;

pub fn send_video_best_effort(clip: &Path, caption: &str) {
    let Some(creds) = load_creds::load_creds() else {
        eprintln!("telegram skipped (no creds): clip={}", clip.display());
        return;
    };
    match send_video_file::send_video_file(&creds, clip, caption) {
        Ok(message) => println!(
            "telegram video sent: message_id={message} clip={}",
            clip.display()
        ),
        Err(err) => eprintln!("telegram send failed: {err} clip={}", clip.display()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::send_video_best_effort;

    #[test]
    fn test_usage() {
        send_video_best_effort(Path::new("/nonexistent-clip.mp4"), "smoke");
    }
}
