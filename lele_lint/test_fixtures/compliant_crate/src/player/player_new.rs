use crate::player;

pub fn new() -> player::Player {
    player::Player::default()
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn test_usage() {
        let p = new();
        assert_eq!(p.health, 100);
    }
}
