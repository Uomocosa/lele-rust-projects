use bevy::prelude::*;

use super::super::menu_root::MenuRoot;

pub fn despawn_menu(mut commands: Commands, menus: Query<Entity, With<MenuRoot>>) {
    for entity in &menus {
        commands.entity(entity).despawn();
    }
}
// no test_usage necessary
