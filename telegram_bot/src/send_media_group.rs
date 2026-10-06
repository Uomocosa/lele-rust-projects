use reqwest::blocking::multipart::{Form, Part};

use crate::Creds;
use crate::Error;
use crate::MediaItem;
use crate::MediaKind;
use crate::media_manifest;
use crate::message_ids;

const MIN_ITEMS: usize = 2;
const MAX_ITEMS: usize = 10;
const MIN_BYTES: usize = 1024;
const PNG_MAGIC: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

/// Sends 2..=10 photos/videos as one album message. Only the first item carries
/// the caption, which Telegram shows once under the whole album.
///
/// # Errors
/// Returns an error if the item count is outside 2..=10, a file cannot be read,
/// is too small, or is not a real png/mp4, the request fails, or Telegram
/// answers with a non-success status or `ok:false`.
pub fn send_media_group(
    creds: &Creds,
    items: &[MediaItem],
    caption: &str,
) -> Result<Vec<String>, Error> {
    if !(MIN_ITEMS..=MAX_ITEMS).contains(&items.len()) {
        return Err(Error::BadAlbumSize { count: items.len() });
    }
    let token = creds.token.as_str();
    let mut form = Form::new().text("chat_id", creds.chat_id.clone());
    for (index, item) in items.iter().enumerate() {
        let bytes = read_item(item)?;
        let (file_name, mime) = match item.kind {
            MediaKind::Photo => ("preview.png", "image/png"),
            MediaKind::Video => ("clip.mp4", "video/mp4"),
        };
        let part = Part::bytes(bytes).file_name(file_name);
        form = form.part(
            format!("file{index}"),
            part.mime_str(mime).map_err(|err| Error::Request {
                op: "sendMediaGroup",
                message: err.to_string(),
            })?,
        );
    }
    form = form.text("media", media_manifest::media_manifest(items, caption));
    let url = format!("https://api.telegram.org/bot{token}/sendMediaGroup");
    let response = reqwest::blocking::Client::new()
        .post(&url)
        .multipart(form)
        .send()
        .map_err(|err| Error::Request {
            op: "sendMediaGroup",
            message: err.to_string(),
        })?;
    let status = response.status();
    let status_text = status.to_string();
    let body = response.text().unwrap_or_default();
    if !status.is_success() {
        let snippet: String = body.chars().take(500).collect();
        return Err(Error::Rejected {
            op: "sendMediaGroup",
            status: status_text,
            snippet,
        });
    }
    message_ids::message_ids("sendMediaGroup", &status_text, &body)
}

// needed helper: same size and magic checks as the single-file senders, so a bad file names itself
fn read_item(item: &MediaItem) -> Result<Vec<u8>, Error> {
    let display = item.path.display().to_string();
    let bytes = std::fs::read(item.path).map_err(|source| Error::Read {
        path: display.clone(),
        source,
    })?;
    let kind = match item.kind {
        MediaKind::Photo => "preview",
        MediaKind::Video => "clip",
    };
    if bytes.len() < MIN_BYTES {
        return Err(Error::TooSmall {
            kind,
            path: display,
            bytes: bytes.len(),
        });
    }
    match item.kind {
        MediaKind::Photo if !bytes.starts_with(&PNG_MAGIC) => Err(Error::NotPng { path: display }),
        MediaKind::Video if !has_ftyp(&bytes) => Err(Error::NotMp4 { path: display }),
        _ => Ok(bytes),
    }
}

fn has_ftyp(bytes: &[u8]) -> bool {
    bytes.windows(4).any(|window| window == b"ftyp")
}

#[cfg(test)]
mod tests {
    use super::send_media_group;
    use crate::{Creds, MediaItem, MediaKind};
    use std::path::Path;

    #[test]
    fn test_usage() {
        let creds = Creds {
            token: "token".to_string(),
            chat_id: "chat".to_string(),
        };
        let one = [MediaItem {
            kind: MediaKind::Photo,
            path: Path::new("/nonexistent-a.png"),
        }];
        assert!(send_media_group(&creds, &one, "c").is_err());
        let two = [one[0], one[0]];
        let err = send_media_group(&creds, &two, "c").unwrap_err();
        assert!(err.to_string().contains("nonexistent-a.png"));
    }
}
