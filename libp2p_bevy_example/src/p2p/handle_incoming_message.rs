use libp2p::PeerId;

use crate::p2p::Message;
use crate::sync::resource::RemoteInputBuffer;

pub fn handle_incoming_message(
    remote_buffer: &mut RemoteInputBuffer,
    peer_id: PeerId,
    msg: Message,
) {
    use tracing::debug;

    match msg {
        Message::PlayerInput { tick, input } => {
            debug!("Received player input from {} for tick {}", peer_id, tick);
            remote_buffer.push(peer_id, tick, input);
        }
        Message::JoinRequest { peer_id } => {
            tracing::info!("Received join request from: {}", peer_id);
        }
        Message::Accept { peer_id } => {
            tracing::info!("Join accepted for: {}", peer_id);
        }
        Message::Reject { peer_id } => {
            tracing::info!("Join rejected for: {}", peer_id);
        }
        Message::PlayerJoined { peer_id } => {
            tracing::info!("Player joined: {}", peer_id);
        }
        Message::PlayerLeft { peer_id } => {
            tracing::info!("Player left: {}", peer_id);
        }
        Message::Ping { .. } => {
            debug!("Received Ping from {}", peer_id);
        }
        Message::Pong { .. } => {
            debug!("Received Pong from {}", peer_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::p2p;
    use crate::p2p::Message;
    use crate::p2p::PlayerInputData;
    use crate::sync::resource::RemoteInputBuffer;
    use libp2p::PeerId;

    #[test]
    fn test_usage() -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = RemoteInputBuffer::default();
        let peer_id = PeerId::random();
        let input = PlayerInputData::from_bools(true, false, false, false);

        p2p::handle_incoming_message(
            &mut buffer,
            peer_id,
            Message::PlayerInput { tick: 5, input },
        );

        let retrieved = match buffer.get(&peer_id, 5) {
            Some(val) => val,
            None => return Err("Should retrieve pushed input".into()),
        };
        assert!(retrieved.left);
        Ok(())
    }

    #[test]
    fn test_ping_does_not_push() {
        let mut buffer = RemoteInputBuffer::default();
        let peer_id = PeerId::random();
        p2p::handle_incoming_message(&mut buffer, peer_id, Message::Ping { peer_id });
        let retrieved = buffer.get(&peer_id, 0);
        assert!(retrieved.is_none());
    }
}
