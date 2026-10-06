use std::path::Path;

use crate::Error;
use crate::deliver;
use crate::deliver::basic::enums::Media;
use crate::deliver::basic::structs::Capture;

/// Sends one capture on its own, or 2+ as a single Telegram album. Returns how many were sent.
pub fn send_group(items: &[&Capture], caption: &str) -> Result<usize, Error> {
    let [only] = items else {
        return send_album(items, caption);
    };
    let sent = deliver::send_one::send_one(&only.path, caption)?;
    Ok(usize::from(sent))
}

// needed helper: albums need credentials too, and absent ones leave artifacts on disk only
fn send_album(items: &[&Capture], caption: &str) -> Result<usize, Error> {
    let Some(creds) = telegram_bot::load_creds() else {
        tracing::warn!("{}", crate::__basic__::constants::NO_CREDENTIALS);
        return Ok(0);
    };
    let media: Vec<telegram_bot::MediaItem> = items
        .iter()
        .map(|capture| telegram_bot::MediaItem {
            kind: media_kind(capture.artifact.media),
            path: Path::new(&capture.path),
        })
        .collect();
    telegram_bot::send_media_group(&creds, &media, caption)
        .map_err(|error| Error::Telegram(error.to_string()))?;
    Ok(items.len())
}

const fn media_kind(media: Media) -> telegram_bot::MediaKind {
    match media {
        Media::Png => telegram_bot::MediaKind::Photo,
        Media::Mp4 => telegram_bot::MediaKind::Video,
    }
}
// no test_usage necessary
