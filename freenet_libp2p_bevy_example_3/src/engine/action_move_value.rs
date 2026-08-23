use crate::engine;

pub fn move_value(action: engine::Action) -> f32 {
    match action.direction {
        engine::Direction::Center => 0.0,
        engine::Direction::Left => -1.0,
        engine::Direction::Right => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use crate::engine;

    use super::move_value;

    #[test]
    fn test_usage() {
        assert_eq!(
            move_value(engine::Action {
                direction: engine::Direction::Right,
                jump: false,
            }),
            1.0
        );
        assert_eq!(
            move_value(engine::Action {
                direction: engine::Direction::Left,
                jump: false,
            }),
            -1.0
        );
        assert_eq!(move_value(engine::Action::default()), 0.0);
    }
}
