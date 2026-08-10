use bevy::prelude::Component;
use derive_more::Deref;

use crate::boxes;

#[derive(Component, Debug, Clone, Copy, Deref)]
pub struct Player(pub boxes::PlayerId);
