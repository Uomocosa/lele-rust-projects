use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    #[default]
    Center,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::Direction;

    #[test]
    fn test_usage() {
        assert_eq!(Direction::default(), Direction::Center);
        assert!(Direction::Left != Direction::Right);
    }
}
