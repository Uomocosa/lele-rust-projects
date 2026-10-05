use std::ffi::OsStr;
use std::path::Path;

use crate::__basic__;
use crate::Error;

pub fn send_one(path: &Path, caption: &str) -> Result<bool, Error> {
    let Some(creds) = telegram_bot::load_creds() else {
        tracing::warn!("{}", __basic__::constants::NO_CREDENTIALS);
        return Ok(false);
    };
    let uploaded = if is_video(path) {
        telegram_bot::send_video_file(&creds, path, caption)
    } else {
        telegram_bot::send_photo_file(&creds, path, caption)
    };
    uploaded.map_err(|error| Error::Telegram(error.to_string()))?;
    Ok(true)
}

// needed helper: extension-based media choice, so the zed task needs only a path
fn is_video(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str) == Some("mp4")
}

// no test_usage necessary
