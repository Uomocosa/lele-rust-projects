use std::path::PathBuf;

use crate::Error;
use crate::deliver;
use crate::deliver::basic::enums::Media;
use crate::deliver::basic::structs::{Artifact, Capture, Manifest};
use crate::scene::Kind;

pub fn collect(
    scene: &str,
    kind: Kind,
    rendered: &[(PathBuf, String, String)],
    previous: Option<&Manifest>,
) -> Result<Vec<Capture>, Error> {
    let mut captures = Vec::new();
    for (path, label, pixels) in rendered {
        let artifact = Artifact {
            scene: scene.to_string(),
            label: label.clone(),
            fingerprint: deliver::fingerprint_of::fingerprint_of(scene, label),
            pixel_hash: pixels.clone(),
            media: Media::Png,
            kind,
        };
        let status = deliver::state_status::state_status(previous, &artifact);
        captures.push(Capture {
            path: path.clone(),
            artifact,
            status,
        });
    }
    Ok(captures)
}
// no test_usage necessary
