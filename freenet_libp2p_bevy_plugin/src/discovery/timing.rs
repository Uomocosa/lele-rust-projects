#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    pub tick_secs: u64,
    pub republish_secs: u64,
    pub presence_ttl_secs: u64,
    pub lobby_secs: u64,
    pub redial_secs: u64,
    pub hello_secs: u64,
    pub member_grace_secs: u64,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            tick_secs: 1,
            republish_secs: 30,
            presence_ttl_secs: 120,
            lobby_secs: 5,
            redial_secs: 2,
            hello_secs: 5,
            member_grace_secs: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Timing;

    #[test]
    fn test_usage() {
        let timing = Timing::default();
        assert_eq!(timing.tick_secs, 1);
        assert!(timing.republish_secs > timing.tick_secs);
    }
}
