use bevy::prelude::Component;

use crate::boxes;

#[derive(Component, Debug, Clone, Copy)]
pub struct Player {
    pub id: boxes::PlayerId,
}
