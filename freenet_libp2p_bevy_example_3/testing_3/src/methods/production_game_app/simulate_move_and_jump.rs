use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;

use crate::structs;

/// Like `simulate_move`, but also holds jump: a horizontal-only key can't clear another
/// box blocking the path, since the two colliders just push back against each other.
pub fn simulate_move_and_jump(
    this: &mut structs::ProductionGameApp,
    direction: KeyCode,
    frames: u32,
) {
    {
        let mut keyboard = this.app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(direction);
        keyboard.press(KeyCode::Space);
    }
    for _ in 0..frames {
        this.app.update();
    }
    {
        let mut keyboard = this.app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.release(direction);
        keyboard.release(KeyCode::Space);
    }
    this.app.update();
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
