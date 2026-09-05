use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::frame::Frame;

#[must_use]
pub fn verify_frame(frame: &Frame) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(&frame.author) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(&frame.sig) else {
        return false;
    };
    let msg =
        bincode::serialize(&(frame.seq, frame.author, frame.prev, frame.next)).unwrap_or_default();
    vk.verify(&msg, &sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::verify_frame;
    use crate::frame_sign_frame::sign_frame;
    use ed25519_dalek::SigningKey;

    #[test]
    fn test_usage() {
        let mut seed = [7u8; 32];
        seed[0] = 2;
        let sk = SigningKey::from_bytes(&seed);
        let f = sign_frame(&sk, 1, b'a', b'b');
        assert!(verify_frame(&f));
    }
}
