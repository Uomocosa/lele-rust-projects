use bevy::prelude::Component;
use derive_more::Deref;

use crate::engine;

#[derive(Component, Debug, Clone, Copy, Deref)]
pub struct Player(pub engine::PlayerId);

#[cfg(test)]
mod tests {
    use super::Player;

    #[test]
    fn test_usage() {
        let player = Player([3; 32]);
        assert_eq!(*player, [3; 32]);
    }
}
