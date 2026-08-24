use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;

use crate::structs;

pub fn press_key(this: &mut structs::ProductionGameApp, key: KeyCode) {
    this.app
        .world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
}
// no test_usage necessary - needs a live embedded freenet node, exercised by tests/
