use std::path::PathBuf;

use crate::preview::basic::constants::{
    DEFAULT_HEIGHT, DEFAULT_LEAD_IN_FRAMES, DEFAULT_MAX_CAPTURE_FRAMES,
    DEFAULT_MIN_DISTINCT_COLORS, DEFAULT_WARMUP_FRAMES, DEFAULT_WIDTH,
};

pub struct Config {
    pub width: u32,
    pub height: u32,
    pub warmup_frames: u32,
    pub lead_in_frames: u32,
    pub max_capture_frames: u32,
    pub min_distinct_colors: usize,
    pub out_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            warmup_frames: DEFAULT_WARMUP_FRAMES,
            lead_in_frames: DEFAULT_LEAD_IN_FRAMES,
            max_capture_frames: DEFAULT_MAX_CAPTURE_FRAMES,
            min_distinct_colors: DEFAULT_MIN_DISTINCT_COLORS,
            out_dir: std::env::temp_dir().join("lele-bevy-preview"),
        }
    }
}

// no test_usage necessary
