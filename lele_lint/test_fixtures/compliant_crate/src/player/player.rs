use super::player_new;
use super::player_with_name;

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

#[rustfmt::skip]
impl Player {
    pub fn new() -> Self { player_new::new() }

    pub fn with_name(self, name: String) -> Self { player_with_name::with_name(self, name) }
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