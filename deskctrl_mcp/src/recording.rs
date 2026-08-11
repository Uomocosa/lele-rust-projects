use std::{process::Child, time::SystemTime};

/// An in-progress ffmpeg screen recording started by `record_video` (or auto-started at session
/// start). Lives on the server so both the tool and the session-end hook can stop it.
pub struct Recording {
    pub path: String,
    pub child: Child,
    pub started: SystemTime,
    pub target: String,
}
