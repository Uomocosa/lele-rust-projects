use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::*;

use crate::discovery;

/// # Errors
/// Returns `Error` if the directory publish update fails.
pub fn publish_room(
    directory: &discovery::Directory,
    room: &str,
    params: &[u8],
    peer_id: &str,
    addrs: &[String],
) -> Result<(), discovery::Error> {
    let mut single = discovery::DirectoryState::new();
    single.insert(
        room.to_string(),
        discovery::DirectoryEntry {
            params: params.to_vec(),
            peer_id: peer_id.to_string(),
            addrs: addrs.to_vec(),
            updated_at: now_secs(),
        },
    );
    let merged = discovery::merge_directory(directory.slots.clone(), single);
    let update_req = ContractRequest::Update {
        key: directory.contract_key,
        data: UpdateData::State(State::from(bincode::serialize(&merged)?)),
    };
    directory
        .client
        .send(&ClientRequest::ContractOp(update_req))?;
    Ok(())
}

// needed helper: seconds since the unix epoch for entry timestamps
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}
// no test_usage necessary
