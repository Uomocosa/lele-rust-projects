use libp2p::identity::Keypair;

#[must_use]
pub fn libp2p_keypair_from_seed(seed: &[u8; 32]) -> Option<Keypair> {
    Keypair::ed25519_from_bytes(*seed).ok()
}

#[cfg(test)]
mod tests {
    use super::libp2p_keypair_from_seed;

    #[test]
    fn test_usage() {
        let _ = libp2p_keypair_from_seed(&[3u8; 32]);
    }
}
