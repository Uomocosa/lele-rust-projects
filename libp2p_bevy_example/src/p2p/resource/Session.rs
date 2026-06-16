use bevy::prelude::Resource;

use crate::p2p::NetworkEvent;
use crate::p2p::Swarm;

#[derive(Resource)]
pub struct Session {
    pub swarm: Swarm,
    pub event_receiver: tokio::sync::mpsc::Receiver<NetworkEvent>,
}
