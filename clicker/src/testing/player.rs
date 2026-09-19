use derive_more::{Deref, From};

use super::player_new;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, From)]
pub struct Player(pub u64);

#[rustfmt::skip]
impl Player {
    #[must_use]
    pub const fn new(id: u64) -> Self { player_new::new(id) }
}

#[cfg(test)]
mod tests {
    use super::Player;

    #[test]
    fn test_usage() {
        let player = Player(3);
        assert_eq!(*player, 3);
        assert_eq!(Player::new(3), player);
        assert_eq!(Player::from(3), player);
    }
}
