use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::*;

use crate::discovery;

/// # Errors
/// Returns `Error` if the roster announce update fails.
pub fn announce(roster: &discovery::Roster) -> Result<(), discovery::Error> {
    let entry = discovery::PeerEntry {
        peer_id: roster.peer_id.clone(),
        addrs: roster.addrs.clone(),
        updated_at: now_secs(),
    };
    let mut single = discovery::RosterState::new();
    single.insert(roster.own, entry);
    let update_req = ContractRequest::Update {
        key: roster.contract_key,
        data: UpdateData::State(State::from(bincode::serialize(&single)?)),
    };
    roster.client.send(&ClientRequest::ContractOp(update_req))?;
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
