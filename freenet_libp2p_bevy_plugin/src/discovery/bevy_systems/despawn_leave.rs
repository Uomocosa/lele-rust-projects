use bevy::prelude::*;

use super::super::leave_root::LeaveRoot;

pub fn despawn_leave(mut commands: Commands, leaves: Query<Entity, With<LeaveRoot>>) {
    for entity in &leaves {
        commands.entity(entity).despawn();
    }
}
// no test_usage necessary
