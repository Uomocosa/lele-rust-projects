use super::stopped_recording_caption;

pub struct StoppedRecording {
    pub path: String,
    pub duration_secs: u64,
    pub size_bytes: u64,
    pub target: String,
}

#[rustfmt::skip]
impl StoppedRecording {
    pub fn caption(&self) -> String { stopped_recording_caption::caption(self) }
}

// no test_usage necessary
