use serde_json::{Value, json};

use crate::MediaItem;
use crate::MediaKind;
use crate::truncate_caption;

/// Builds the `media` JSON array of a `sendMediaGroup` call.
///
/// Item `i` is uploaded as multipart part `file{i}`. Telegram shows one caption
/// per album, taken from the first item, so only item 0 carries it.
#[must_use]
pub fn media_manifest(items: &[MediaItem], caption: &str) -> String {
    let entries: Vec<Value> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let kind = match item.kind {
                MediaKind::Photo => "photo",
                MediaKind::Video => "video",
            };
            let mut entry = serde_json::Map::new();
            entry.insert("type".to_string(), json!(kind));
            entry.insert("media".to_string(), json!(format!("attach://file{index}")));
            if index == 0 {
                entry.insert(
                    "caption".to_string(),
                    Value::String(truncate_caption(caption)),
                );
            }
            Value::Object(entry)
        })
        .collect();
    Value::Array(entries).to_string()
}

#[cfg(test)]
mod tests {
    use super::media_manifest;
    use crate::{MediaItem, MediaKind};
    use serde_json::Value;
    use std::path::Path;

    #[test]
    fn test_usage() {
        let items = [
            MediaItem {
                kind: MediaKind::Photo,
                path: Path::new("a.png"),
            },
            MediaItem {
                kind: MediaKind::Video,
                path: Path::new("b.mp4"),
            },
        ];
        let parsed: Value = serde_json::from_str(&media_manifest(&items, "hello")).unwrap();
        let entries = parsed.as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["type"], "photo");
        assert_eq!(entries[0]["media"], "attach://file0");
        assert_eq!(entries[0]["caption"], "hello");
        assert_eq!(entries[1]["type"], "video");
        assert_eq!(entries[1]["media"], "attach://file1");
        assert!(entries[1].get("caption").is_none());
    }
}
