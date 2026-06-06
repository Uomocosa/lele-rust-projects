use bevy::prelude::Message;

#[derive(Message, Debug, Clone)]
pub enum Event {
    ContractResponse { contract_id: String, data: Vec<u8> },
    PeerConnected { peer_id: String },
    PeerDisconnected { peer_id: String },
    Error { description: String },
}
