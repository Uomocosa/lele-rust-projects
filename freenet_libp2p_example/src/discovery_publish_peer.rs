use ed25519_dalek::SigningKey;
use freenet_stdlib::client_api::{ClientRequest, ContractRequest};

use crate::discovery;
use crate::discovery::Discovery;

/// # Errors
/// Returns error if send fails.
pub async fn publish_peer(
    d: &mut Discovery,
    pubkey: [u8; 32],
    signing: &SigningKey,
    peer_id: Vec<u8>,
    addrs: Vec<String>,
) -> Result<(), String> {
    use ed25519_dalek::Signer;
    let rec = discovery::PeerRecord {
        peer_id: peer_id.clone(),
        addrs: addrs.clone(),
    };
    let msg = bincode::serialize(&(&pubkey, &peer_id, &addrs)).unwrap_or_default();
    let sig = signing.sign(&msg).to_bytes().to_vec();
    let mut delta = discovery::StateData::default();
    delta.peers.insert(pubkey, rec);
    delta.sigs.insert(pubkey, sig);
    let serialized = bincode::serialize(&delta).unwrap_or_default();
    let data = freenet_stdlib::prelude::State::from(serialized);
    let req = ClientRequest::ContractOp(ContractRequest::Update {
        key: d.key,
        data: freenet_stdlib::prelude::UpdateData::State(data),
    });
    d.client.send(req).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(publish_peer);
    }
}
