use std::io::Write;
use std::path::{Path, PathBuf};

use libp2p::identity::Keypair;

// needed helper:
fn identity_file_path(dir_override: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(dir) = dir_override {
        return Some(dir.join("identity.bin"));
    }
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home).join(".local/share/freenet-libp2p-bevy-plugin/identity.bin")
    })
}

// needed helper:
/// Writes to a temp file in the same directory and renames it over `path`, so a concurrent reader
/// sees either the previous complete file or the new complete file — never a partial one.
fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp_path = path.with_extension("tmp");
    let mut file = std::fs::File::create(&tmp_path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    std::fs::rename(&tmp_path, path)
}

pub fn load_or_create_keypair(dir_override: Option<PathBuf>) -> Keypair {
    let Some(path) = identity_file_path(dir_override) else {
        tracing::warn!(target: "p2p", "no HOME set, using ephemeral identity");
        return Keypair::generate_ed25519();
    };
    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(keypair) = Keypair::from_protobuf_encoding(&bytes) {
            tracing::info!(
                target: "p2p",
                ?path,
                peer_id = %keypair.public().to_peer_id(),
                "loaded persisted identity"
            );
            return keypair;
        }
        tracing::warn!(target: "p2p", ?path, "unreadable identity file, regenerating");
    }
    let keypair = Keypair::generate_ed25519();
    tracing::info!(
        target: "p2p",
        ?path,
        peer_id = %keypair.public().to_peer_id(),
        "generated fresh identity"
    );
    match keypair.to_protobuf_encoding() {
        Ok(bytes) => {
            if let Some(parent) = path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                tracing::warn!(target: "p2p", ?path, error = %e, "failed to create identity dir");
                return keypair;
            }
            if let Err(e) = atomic_write(&path, &bytes) {
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
        let dir = std::env::temp_dir().join(format!(
            "freenet_libp2p_bevy_plugin_test_{}",
            std::process::id()
        ));
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
        let dir_a = std::env::temp_dir().join(format!(
            "freenet_libp2p_bevy_plugin_test_a_{}",
            std::process::id()
        ));
        let dir_b = std::env::temp_dir().join(format!(
            "freenet_libp2p_bevy_plugin_test_b_{}",
            std::process::id()
        ));
        let a = load_or_create_keypair(Some(dir_a.clone()));
        let b = load_or_create_keypair(Some(dir_b.clone()));
        assert_ne!(a.public().to_peer_id(), b.public().to_peer_id());
        std::fs::remove_dir_all(&dir_a).ok();
        std::fs::remove_dir_all(&dir_b).ok();
    }
}
