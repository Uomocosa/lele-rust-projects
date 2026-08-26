pub struct Config {
    pub instances: usize,
    pub release: bool,
    pub mode: String,
    pub repeats: usize,
    pub timeout_secs: u64,
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
            timeout_secs: 600,
            settle_secs: 600,
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
