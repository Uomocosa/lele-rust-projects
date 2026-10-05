use std::path::Path;

use crate::deliver;
use crate::deliver::basic::structs::Manifest;

#[must_use]
pub fn load_previous(dir: &Path) -> Option<Manifest> {
    let text = std::fs::read_to_string(dir.join(deliver::basic::constants::MANIFEST_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}
// no test_usage necessary
