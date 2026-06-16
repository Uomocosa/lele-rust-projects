use crate::boxes::component::Position;

pub fn zero() -> Position {
    Position { x: 0.0, y: 0.0 }
}

#[cfg(test)]
mod tests {
    use crate::boxes::component::Position;

    #[test]
    fn test_usage() {
        let zero = Position::zero();
        assert_eq!(zero.x, 0.0);
        assert_eq!(zero.y, 0.0);
    }
}
