use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;

use crate::structs;

/// Drives the real input -> physics -> `p2p::bevy_systems::send_snapshot` pipeline (not a
/// synthetic position write), matching how a player actually moves a box.
pub fn simulate_move(this: &mut structs::ProductionGameApp, direction: KeyCode, frames: u32) {
    this.app
        .world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(direction);
    for _ in 0..frames {
        this.app.update();
    }
    this.app
        .world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(direction);
    this.app.update();
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
