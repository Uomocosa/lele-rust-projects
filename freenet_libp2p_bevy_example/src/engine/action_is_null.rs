use crate::engine;

pub fn is_null(action: engine::Action) -> bool {
    action == engine::Action::default()
}

#[cfg(test)]
mod tests {
    use crate::engine;

    use super::is_null;

    #[test]
    fn test_usage() {
        assert!(is_null(engine::Action::default()));
        assert!(!is_null(engine::Action {
            direction: engine::Direction::Right,
            jump: true,
        }));
    }
}
