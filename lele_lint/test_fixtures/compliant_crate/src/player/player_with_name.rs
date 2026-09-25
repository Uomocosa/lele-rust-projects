use crate::player;

pub fn with_name(base: player::Player, name: String) -> player::Player {
    player::Player { name, ..base }
}

#[cfg(test)]
mod tests {
    use super::with_name;
    use crate::player::Player;

    #[test]
    fn test_usage() {
        let p = Player::default();
        let p = with_name(p, "Alice".into());
        assert_eq!(&p.name, "Alice");
    }
}