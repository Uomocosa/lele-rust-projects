use crate::p2p::Config;

pub fn with_manual_dial(mut cfg: Config, enabled: bool) -> Config {
    cfg.enable_manual_dial = enabled;
    cfg
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::default().with_manual_dial(false);
        assert!(!config.enable_manual_dial);
    }
}
