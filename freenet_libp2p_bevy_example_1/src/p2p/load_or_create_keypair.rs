use std::path::PathBuf;

use libp2p::identity::Keypair;

// needed helper:
fn identity_file_path(dir_override: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(dir) = dir_override {
        return Some(dir.join("identity.bin"));
    }
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".local/share/bevy_freenet/identity.bin"))
}

pub fn load_or_create_keypair(dir_override: Option<PathBuf>) -> Keypair {
    let Some(path) = identity_file_path(dir_override) else {
        tracing::warn!(target: "p2p", "no HOME set, using ephemeral identity");
        return Keypair::generate_ed25519();
    };
    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(keypair) = Keypair::from_protobuf_encoding(&bytes) {
            return keypair;
        }
        tracing::warn!(target: "p2p", ?path, "unreadable identity file, regenerating");
    }
    let keypair = Keypair::generate_ed25519();
    match keypair.to_protobuf_encoding() {
        Ok(bytes) => {
            if let Some(parent) = path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                tracing::warn!(target: "p2p", ?path, error = %e, "failed to create identity dir");
                return keypair;
            }
            if let Err(e) = std::fs::write(&path, bytes) {
                tracing::warn!(target: "p2p", ?path, error = %e, "failed to persist identity");
            }
        }
        Err(e) => tracing::warn!(target: "p2p", error = %e, "failed to encode identity"),
    }
    keypair
}

#[cfg(test)]
mod tests {
    use libp2p::identity::Keypair;

    use super::load_or_create_keypair;

    #[test]
    fn test_usage() {
        let dir = std::env::temp_dir().join(format!("bevy_freenet_test_{}", std::process::id()));
        let keypair = load_or_create_keypair(Some(dir.clone()));
        let bytes = keypair.to_protobuf_encoding().unwrap();
        let restored = Keypair::from_protobuf_encoding(&bytes).unwrap();
        assert_eq!(
            keypair.public().to_peer_id(),
            restored.public().to_peer_id()
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn different_dirs_yield_different_identities() {
        let dir_a = std::env::temp_dir().join(format!("bevy_freenet_test_a_{}", std::process::id()));
        let dir_b = std::env::temp_dir().join(format!("bevy_freenet_test_b_{}", std::process::id()));
        let a = load_or_create_keypair(Some(dir_a.clone()));
        let b = load_or_create_keypair(Some(dir_b.clone()));
        assert_ne!(a.public().to_peer_id(), b.public().to_peer_id());
        std::fs::remove_dir_all(&dir_a).ok();
        std::fs::remove_dir_all(&dir_b).ok();
    }
}
