use bevy::prelude::*;
use libp2p::PeerId;

#[derive(Component, Debug, Clone)]
pub struct Player {
    pub peer_id: PeerId,
    pub is_local: bool,
}
