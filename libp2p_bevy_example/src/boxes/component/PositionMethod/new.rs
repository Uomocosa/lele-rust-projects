use crate::boxes::component::Position;

pub fn new(x: f32, y: f32) -> Position {
    Position { x, y }
}

#[cfg(test)]
mod tests {
    use crate::boxes::component::Position;

    #[test]
    fn test_usage() {
        let pos = Position::new(10.0, 20.0);
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
    }
}
