use bevy::prelude::Message;

use crate::net_id;
use crate::p2p;

#[derive(Message, Clone, Debug)]
pub struct IncomingSnapshot<T> {
    pub from: net_id::NetworkId,
    pub snapshot: p2p::Snapshot<T>,
}
