use crate::p2p::PlayerInputData;
use crate::sync::resource::RemoteInputBuffer;
use libp2p::PeerId;

pub fn get(buffer: &RemoteInputBuffer, peer_id: &PeerId, tick: u64) -> Option<PlayerInputData> {
    buffer.inputs.get(peer_id).and_then(|inputs| {
        inputs
            .iter()
            .find(|(t, _)| *t == tick)
            .map(|(_, input)| input.clone())
    })
}

#[cfg(test)]
mod tests {
    use crate::p2p::PlayerInputData;
    use crate::sync::resource::RemoteInputBuffer;
    use libp2p::PeerId;

    #[test]
    fn test_usage() {
        let mut buffer = RemoteInputBuffer::default();
        let peer_id = PeerId::random();
        let input = PlayerInputData::from_bools(true, false, false, false);
        buffer.push(peer_id, 1, input);
        let retrieved = buffer.get(&peer_id, 1);
        assert!(retrieved.is_some());
        if let Some(result) = retrieved {
            assert!(result.left);
        }
    }

    #[test]
    fn test_returns_none_for_missing_tick() {
        let buffer = RemoteInputBuffer::default();
        let peer_id = PeerId::random();
        assert!(buffer.get(&peer_id, 0).is_none());
    }
}
