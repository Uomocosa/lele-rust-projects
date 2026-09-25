use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::*;

use crate::discovery;

pub fn publish_room(
    directory: &discovery::link::CatalogClient,
    room: &discovery::params::RoomName,
    params: &discovery::params::ContractParams,
    peer_id: &discovery::params::RemotePeerId,
    addrs: &[String],
) -> Result<(), discovery::Error> {
    let mut single = discovery::directory::RoomCatalog::new();
    single.insert(
        room.clone(),
        discovery::directory::RoomPayload {
            params: params.clone(),
            peer_id: peer_id.clone(),
            addrs: addrs.to_vec(),
            updated_at: discovery::params::EpochSecs(now_secs()),
        },
    );
    let merged = discovery::directory::merge_directory(directory.slots.clone(), single);
    let update_req = ContractRequest::Update {
        key: directory.contract_key,
        data: UpdateData::State(State::from(bincode::serialize(&merged)?)),
    };
    directory
        .client
        .send(&ClientRequest::ContractOp(update_req))?;
    Ok(())
}

// needed helper: seconds since the unix epoch for payload timestamps
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert!(super::now_secs() > 0);
    }
}
