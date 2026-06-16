use bevy::input::ButtonInput;
use bevy::prelude::KeyCode;

use crate::p2p::PlayerInputData;

pub fn collect_input(button_input: &ButtonInput<KeyCode>) -> PlayerInputData {
    let left = button_input.pressed(KeyCode::ArrowLeft) || button_input.pressed(KeyCode::KeyD);
    let right = button_input.pressed(KeyCode::ArrowRight);
    let up = button_input.pressed(KeyCode::ArrowUp) || button_input.pressed(KeyCode::KeyW);
    let jump = button_input.pressed(KeyCode::Space);

    tracing::trace!(target: "player_input", left, right, up, jump);

    PlayerInputData::from_bools(left, right, up, jump)
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonInput;
    use bevy::prelude::KeyCode;

    #[test]
    fn test_usage() {
        let mut button_input = ButtonInput::default();

        button_input.press(KeyCode::ArrowRight);
        let result = super::collect_input(&button_input);
        assert!(result.right, "Right key should register");

        button_input.press(KeyCode::Space);
        let result = super::collect_input(&button_input);
        assert!(result.right, "Right should still be pressed");
        assert!(result.jump, "Jump key should register");
    }
}
