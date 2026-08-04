use super::player::Player;

pub fn with_name(player: Player, name: String) -> Player {
    Player { name, ..player }
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