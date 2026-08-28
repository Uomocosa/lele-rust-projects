pub struct Config {
    pub instances: usize,
    pub release: bool,
    pub mode: String,
    pub repeats: usize,
    /// Hard ceiling: a trial that hasn't reconciled+merged by this time is a failure.
    pub timeout_secs: u64,
    /// Stabilization window: how long the merged condition must hold continuously
    /// before a trial ends early on success.
    pub settle_secs: u64,
    pub clip_secs: u64,
    pub no_video: bool,
    pub no_telegram: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            instances: 2,
            release: true,
            mode: "both".to_string(),
            repeats: 1,
            timeout_secs: 900,
            settle_secs: 30,
            clip_secs: 25,
            no_video: false,
            no_telegram: false,
        }
    }
}

#[rustfmt::skip]
impl Config {
    pub fn modes(&self) -> Vec<&str> {
        match self.mode.as_str() {
            "counter" => vec!["counter"],
            "set" => vec!["set"],
            _ => vec!["counter", "set"],
        }
    }
}

// no test_usage necessary — companion to parse_config
