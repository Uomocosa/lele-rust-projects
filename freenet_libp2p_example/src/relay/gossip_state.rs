use std::collections::HashMap;

use super::gossip_state_insert;
use super::gossip_state_new;
use super::gossip_state_should_accept;
use crate::frame::Frame;

pub struct GossipState {
    pub seen: HashMap<u64, Frame>,
    pub last_next: u8,
}

impl Default for GossipState {
    fn default() -> Self {
        Self::new()
    }
}

#[rustfmt::skip]
impl GossipState {
    #[must_use]
    pub fn new() -> Self { gossip_state_new::new() }
    #[must_use]
    pub fn should_accept(&self, frame: &Frame) -> bool { gossip_state_should_accept::should_accept(self, frame) }
    pub fn insert(&mut self, frame: Frame) { gossip_state_insert::insert(self, frame) }
}

// no test_usage necessary
