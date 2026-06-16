use crate::boxes::component::PlayerInputMethod;
use crate::p2p::PlayerInputData;
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default)]
pub struct PlayerInput {
    pub input: PlayerInputData,
}

#[rustfmt::skip]
impl PlayerInput {
    pub fn new() -> Self { PlayerInputMethod::new() }
}
