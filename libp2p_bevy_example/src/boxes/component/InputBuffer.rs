use crate::p2p::PlayerInputData;
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct InputBuffer {
    pub inputs: Vec<(u64, PlayerInputData)>,
    pub max_size: usize,
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self {
            inputs: Vec::new(),
            max_size: 128,
        }
    }
}
