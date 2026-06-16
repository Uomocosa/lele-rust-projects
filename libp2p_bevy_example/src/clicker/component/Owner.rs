use bevy::prelude::*;
use libp2p::PeerId;

use crate::clicker::component::OwnerMethod;

#[derive(Component, Debug, Clone)]
pub struct Owner {
    pub peer_id: PeerId,
}

#[rustfmt::skip]
impl Owner {
    pub fn new(peer_id: PeerId) -> Self { OwnerMethod::new(peer_id) }
    pub fn is_local(&self, p2p_state: &crate::p2p::resource::PeerState) -> bool { OwnerMethod::is_local(self, p2p_state) }
}
