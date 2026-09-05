use ed25519_dalek::{Signer, SigningKey, VerifyingKey};

use crate::frame::Frame;

#[must_use]
pub fn sign_frame(sk: &SigningKey, seq: u64, prev: u8, next: u8) -> Frame {
    let pk = *VerifyingKey::from(sk).as_bytes();
    let msg = bincode::serialize(&(seq, pk, prev, next)).unwrap_or_default();
    let sig = sk.sign(&msg).to_bytes().to_vec();
    Frame {
        seq,
        prev,
        next,
        author: pk,
        sig,
    }
}

#[cfg(test)]
mod tests {
    use super::sign_frame;
    use crate::frame_verify_frame::verify_frame;
    use ed25519_dalek::SigningKey;

    #[test]
    fn test_usage() {
        let mut seed = [7u8; 32];
        seed[0] = 1;
        let sk = SigningKey::from_bytes(&seed);
        let f = sign_frame(&sk, 0, 0, b'a');
        assert!(verify_frame(&f));
    }
}
