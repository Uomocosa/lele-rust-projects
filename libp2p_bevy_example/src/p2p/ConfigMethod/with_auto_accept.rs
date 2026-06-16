use crate::p2p::Config;

pub fn with_auto_accept(mut cfg: Config, accept: bool) -> Config {
    cfg.auto_accept_join = accept;
    cfg
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;

    #[test]
    fn test_usage() {
        let config = Config::default().with_auto_accept(false);
        assert!(!config.auto_accept_join);
    }
}
