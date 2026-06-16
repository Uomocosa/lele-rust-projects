use crate::boxes::component::Velocity;

pub fn zero() -> Velocity {
    Velocity { x: 0.0, y: 0.0 }
}

#[cfg(test)]
mod tests {
    use crate::boxes::component::Velocity;

    #[test]
    fn test_usage() {
        let zero = Velocity::zero();
        assert_eq!(zero.x, 0.0);
        assert_eq!(zero.y, 0.0);
    }
}
