use bevy::prelude::Resource;

#[derive(Resource, Debug, Default)]
pub struct JoinClock {
    pub clicked_at: Option<std::time::Instant>,
    pub last_new_peer: Option<std::time::Instant>,
}

#[cfg(test)]
mod tests {
    use super::JoinClock;

    #[test]
    fn test_usage() {
        let clock = JoinClock::default();
        assert!(clock.clicked_at.is_none());
        assert!(clock.last_new_peer.is_none());
        let clock = JoinClock {
            clicked_at: Some(std::time::Instant::now()),
            last_new_peer: Some(std::time::Instant::now()),
        };
        assert!(clock.clicked_at.is_some());
        assert!(clock.last_new_peer.is_some());
    }
}
