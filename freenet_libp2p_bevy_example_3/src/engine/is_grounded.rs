use crate::engine;

pub fn is_grounded(center_y: f32) -> bool {
    let ground_top = engine::GROUND_TOP;
    center_y - engine::BOX_SIZE / 2.0 <= ground_top + 1.0
}

#[cfg(test)]
mod tests {
    use super::is_grounded;
    use crate::engine;

    #[test]
    fn test_usage() {
        assert!(is_grounded(engine::SPAWN_Y));
        assert!(!is_grounded(engine::SPAWN_Y + 200.0));
    }
}
