pub struct Config {
    pub instances: usize,
    pub release: bool,
    pub timeout_secs: u64,
    pub settle_secs: u64,
    pub no_video: bool,
    pub no_telegram: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            instances: 3,
            release: true,
            timeout_secs: 480,
            settle_secs: 45,
            no_video: false,
            no_telegram: false,
        }
    }
}

// no test_usage necessary — companion to parse_config
