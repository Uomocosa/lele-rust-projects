use crate::p2p::Config;

pub fn with_max_players(mut cfg: Config, max: usize) -> Config {
    cfg.max_players = Some(max);
    cfg
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::default().with_max_players(4);
        assert_eq!(config.max_players, Some(4));
    }
}
