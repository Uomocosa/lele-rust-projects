use bevy::prelude::Component;
use derive_more::Deref;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct PlayerNo(pub u64);

#[cfg(test)]
mod tests {
    use super::PlayerNo;

    #[test]
    fn test_usage() {
        let player = PlayerNo(2);
        assert_eq!(*player, 2);
    }
}
