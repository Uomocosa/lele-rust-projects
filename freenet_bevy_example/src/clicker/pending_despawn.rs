use bevy::prelude::{Component, Timer};

/// Marks an entity to be despawned once its exit animation finishes.
#[derive(Component)]
pub struct PendingDespawn {
    pub timer: Timer,
}

#[cfg(test)]
mod tests {
    use super::PendingDespawn;
    use bevy::prelude::{Timer, TimerMode};
    use std::time::Duration;

    #[test]
    fn test_usage() {
        let pd = PendingDespawn {
            timer: Timer::new(Duration::from_millis(1), TimerMode::Once),
        };
        assert_eq!(pd.timer.duration(), Duration::from_millis(1));
    }
}
