use super::config::Config;

pub fn parse_config() -> Config {
    let mut cfg = Config::default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "release" => cfg.release = true,
            "dev" => cfg.release = false,
            "--no-video" => cfg.no_video = true,
            "--no-telegram" => cfg.no_telegram = true,
            "--mode" => {
                if let Some(v) = args.get(i + 1) {
                    cfg.mode = v.to_string();
                    i += 1;
                }
            }
            "--repeat" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse::<usize>().ok()) {
                    cfg.repeats = v;
                    i += 1;
                }
            }
            "--timeout" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                    cfg.timeout_secs = v;
                    i += 1;
                }
            }
            "--settle" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                    cfg.settle_secs = v;
                    i += 1;
                }
            }
            "--clip" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                    cfg.clip_secs = v;
                    i += 1;
                }
            }
            _ => {
                if let Ok(n) = arg.parse::<usize>()
                    && n > 0
                {
                    cfg.instances = n;
                }
            }
        }
        i += 1;
    }
    cfg
}

#[cfg(test)]
mod tests {
    use super::parse_config;

    #[test]
    fn test_usage() {
        let cfg = parse_config();
        assert_eq!(cfg.modes(), vec!["counter", "set"]);
        assert_eq!(cfg.instances, 2);
    }
}
