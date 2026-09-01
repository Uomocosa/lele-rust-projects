pub struct Config {
    pub instances: usize,
    pub release: bool,
    pub timeout_secs: u64,
    pub no_video: bool,
    pub no_telegram: bool,
    pub allowed_flicker_secs: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            instances: 3,
            release: false,
            timeout_secs: 420,
            no_video: false,
            no_telegram: false,
            allowed_flicker_secs: 10.0,
        }
    }
}

// no test_usage necessary — companion to parse_config
