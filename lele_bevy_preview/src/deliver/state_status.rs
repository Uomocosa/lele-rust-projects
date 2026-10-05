use crate::deliver::basic::enums::Status;
use crate::deliver::basic::structs::{Artifact, Manifest};

#[must_use]
pub fn state_status(previous: Option<&Manifest>, artifact: &Artifact) -> Status {
    let Some(previous) = previous else {
        return Status::FirstRun;
    };
    let before = previous.artifacts.get(&artifact.fingerprint).or_else(|| {
        previous
            .artifacts
            .values()
            .find(|old| old.scene == artifact.scene && old.label == artifact.label)
    });
    match before {
        None => Status::New,
        Some(old) if old.pixel_hash != artifact.pixel_hash => Status::Changed,
        Some(_) => Status::Same,
    }
}
// no test_usage necessary
