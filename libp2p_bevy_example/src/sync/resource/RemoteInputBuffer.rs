use bevy::prelude::*;
use libp2p::PeerId;
use std::collections::HashMap;

use crate::p2p::PlayerInputData;
use crate::sync::resource::RemoteInputBufferMethod;

#[derive(Resource)]
pub struct RemoteInputBuffer {
    pub(crate) inputs: HashMap<PeerId, Vec<(u64, PlayerInputData)>>,
    pub(crate) max_size: usize,
}

impl Default for RemoteInputBuffer {
    fn default() -> Self {
        Self {
            inputs: HashMap::new(),
            max_size: 256,
        }
    }
}

#[rustfmt::skip]
impl RemoteInputBuffer {
    pub fn get(&self, peer_id: &PeerId, tick: u64) -> Option<PlayerInputData> { RemoteInputBufferMethod::get(self, peer_id, tick) }
    pub fn push(&mut self, peer_id: PeerId, tick: u64, input: PlayerInputData) { RemoteInputBufferMethod::push(self, peer_id, tick, input) }
}
