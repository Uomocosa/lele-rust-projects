use std::fmt::Write;

use crate::deliver;
use crate::deliver::basic::enums::Media;
use crate::deliver::basic::structs::Capture;
use crate::scene::Kind;

/// One caption for a whole album: a tagged header, then one numbered line per item
/// in album order, because Telegram shows a single caption under the grid.
#[must_use]
pub fn caption(kind: Kind, scene: &str, items: &[&Capture]) -> String {
    let mut out = format!("{} {scene}", kind.tag());
    for (index, capture) in items.iter().enumerate() {
        let icon = match capture.artifact.media {
            Media::Png => "🖼",
            Media::Mp4 => "▶",
        };
        let _ = write!(
            out,
            "\n{}. {icon} {} · {}",
            index.saturating_add(1),
            capture.artifact.label,
            deliver::status_word::status_word(capture.status)
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::caption;
    use crate::deliver::basic::enums::{Media, Status};
    use crate::deliver::basic::structs::{Artifact, Capture};
    use crate::scene::Kind;
    use std::path::PathBuf;

    // needed helper: fixture shared by the tests in this file
    fn capture(label: &str, media: Media, status: Status) -> Capture {
        Capture {
            path: PathBuf::from(format!("out/{label}.png")),
            artifact: Artifact {
                scene: String::from("sync_room_list"),
                label: label.to_string(),
                fingerprint: format!("sync_room_list|{label}"),
                pixel_hash: String::from("h"),
                media,
                kind: Kind::System,
            },
            status,
        }
    }

    #[test]
    fn test_usage() {
        let two = capture("two rooms", Media::Png, Status::Same);
        let three = capture("three rooms", Media::Png, Status::Changed);
        let text = caption(Kind::System, "sync_room_list", &[&two, &three]);
        assert_eq!(
            text,
            "[system] sync_room_list\n1. 🖼 two rooms · same\n2. 🖼 three rooms · changed"
        );
        let clip = capture("add", Media::Mp4, Status::FirstRun);
        assert!(caption(Kind::App, "x", &[&clip]).contains("[app] x\n1. ▶ add · first run"));
    }
}
