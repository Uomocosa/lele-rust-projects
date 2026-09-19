use bevy::prelude::Component;

#[derive(Component, Debug, Clone, Copy)]
pub struct PendingReveal {
    pub reveal_at: std::time::Instant,
    pub player: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::PendingReveal;

    #[test]
    fn test_usage() {
        let now = std::time::Instant::now();
        let marker = PendingReveal {
            reveal_at: now,
            player: Some(7),
        };
        assert_eq!(marker.reveal_at, now);
        assert_eq!(marker.player, Some(7));
    }
}
