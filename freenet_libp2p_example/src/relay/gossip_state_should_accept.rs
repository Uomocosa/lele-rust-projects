use crate::frame;
use crate::frame_verify_frame;
use crate::relay;

#[must_use]
pub fn should_accept(state: &relay::gossip_state::GossipState, frame: &frame::Frame) -> bool {
    if !frame_verify_frame::verify_frame(frame) {
        return false;
    }
    if let Some(existing) = state.seen.get(&frame.seq) {
        return existing != frame;
    }
    if frame.seq == 0 {
        return frame.prev == 0;
    }
    if let Some(prev) = state.seen.get(&frame.seq.saturating_sub(1)) {
        return prev.next == frame.prev;
    }
    state.seen.is_empty() && frame.seq == 0
}

#[cfg(test)]
mod tests {
    use super::should_accept;
    use crate::frame_sign_frame::sign_frame;
    use crate::relay;
    use ed25519_dalek::SigningKey;

    #[test]
    fn test_usage() {
        let sk = SigningKey::from_bytes(&[9u8; 32]);
        let f = sign_frame(&sk, 0, 0, b'a');
        let s = relay::gossip_state::GossipState::new();
        assert!(should_accept(&s, &f));
    }
}
