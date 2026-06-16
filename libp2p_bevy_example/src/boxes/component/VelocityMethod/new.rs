use crate::boxes::component::Velocity;

pub fn new(x: f32, y: f32) -> Velocity {
    Velocity { x, y }
}

#[cfg(test)]
mod tests {
    use crate::boxes::component::Velocity;

    #[test]
    fn test_usage() {
        let vel = Velocity::new(5.0, -3.0);
        assert_eq!(vel.x, 5.0);
        assert_eq!(vel.y, -3.0);
    }
}
