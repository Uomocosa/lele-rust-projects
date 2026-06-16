use crate::p2p::PlayerInputData;
use crate::sync::resource::RemoteInputBuffer;
use libp2p::PeerId;

pub fn push(buffer: &mut RemoteInputBuffer, peer_id: PeerId, tick: u64, input: PlayerInputData) {
    let peer_inputs = buffer.inputs.entry(peer_id).or_default();
    if peer_inputs.len() >= buffer.max_size {
        peer_inputs.remove(0);
    }
    peer_inputs.push((tick, input));
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
        if let Some(input) = retrieved {
            assert!(input.left);
        }
    }
}
