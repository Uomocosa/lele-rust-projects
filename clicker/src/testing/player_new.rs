use super::player::Player;

pub const fn new(id: u64) -> Player {
    Player(id)
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn test_usage() {
        assert_eq!(new(7), super::Player(7));
    }
}
