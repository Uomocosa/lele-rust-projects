use atomic_delegate_macros::atomic_delegates;

use crate::player;

pub struct Player {
    pub name: String,
    pub health: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            name: String::new(),
            health: 100,
        }
    }
}

impl Player {
    pub fn new() -> Self { Self::default() }
}

#[atomic_delegates]
impl Player {
    pub fn with_name(self, name: String) -> Self {}
}

#[cfg(test)]
mod tests {
    use crate::player;

    #[test]
    fn test_usage() {
        let p = player::Player::new();
        assert_eq!(p.health, 100);
    }
}
