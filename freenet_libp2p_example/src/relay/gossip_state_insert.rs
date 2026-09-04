use crate::frame;
use crate::relay;

pub fn insert(state: &mut relay::gossip_state::GossipState, frame: frame::Frame) {
    if state.seen.contains_key(&frame.seq) {
        return;
    }
    state.last_next = frame.next;
    state.seen.insert(frame.seq, frame);
}

#[cfg(test)]
mod tests {
    use super::insert;
    use crate::frame_sign_frame::sign_frame;
    use crate::relay;
    use ed25519_dalek::SigningKey;

    #[test]
    fn test_usage() {
        let sk = SigningKey::from_bytes(&[5u8; 32]);
        let f = sign_frame(&sk, 0, 0, b'x');
        let mut s = relay::gossip_state::GossipState::new();
        insert(&mut s, f);
        assert_eq!(s.seen.len(), 1);
    }
}
