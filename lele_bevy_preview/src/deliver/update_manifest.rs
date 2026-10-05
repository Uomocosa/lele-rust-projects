use std::collections::BTreeMap;
use std::path::Path;

use crate::Error;
use crate::deliver;
use crate::deliver::basic::structs::{Capture, Manifest};

pub fn update_manifest(
    dir: &Path,
    crate_name: &str,
    captures: &[Capture],
    clip: Option<&Capture>,
) -> Result<(), Error> {
    let empty = Manifest {
        crate_name: crate_name.to_string(),
        artifacts: BTreeMap::new(),
    };
    let mut merged = deliver::load_previous::load_previous(dir).unwrap_or(empty);
    merged.crate_name = crate_name.to_string();
    for capture in captures.iter().chain(clip) {
        merged.artifacts.insert(
            capture.artifact.fingerprint.clone(),
            capture.artifact.clone(),
        );
    }
    let json =
        serde_json::to_string_pretty(&merged).map_err(|error| Error::Io(error.to_string()))?;
    std::fs::create_dir_all(dir).map_err(|error| Error::Io(error.to_string()))?;
    std::fs::write(dir.join(deliver::basic::constants::MANIFEST_FILE), json)
        .map_err(|error| Error::Io(error.to_string()))
}
// no test_usage necessary
