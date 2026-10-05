use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::deliver;
use crate::deliver::basic::enums::Media;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub scene: String,
    pub label: String,
    pub fingerprint: String,
    pub pixel_hash: String,
    #[serde(default)]
    pub media: Media,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub crate_name: String,
    pub artifacts: BTreeMap<String, Artifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    pub path: PathBuf,
    pub artifact: Artifact,
    pub status: deliver::Status,
}
