use std::time::Duration;

use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::discovery;

const DRAIN_TIMEOUT: Duration = Duration::from_millis(10);

pub async fn poll(
    roster: &mut discovery::link::RosterClient,
) -> Result<Vec<discovery::membership::PeerEntry>, discovery::Error> {
    let mut fresh = Vec::new();
    while let Some(result) = roster.client.recv_response_timeout(DRAIN_TIMEOUT).await {
        match result? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification {
                update, ..
            }) => absorb_update(roster, update, &mut fresh),
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                absorb_bytes(roster, state.as_ref(), &mut fresh);
            }
            _ => {}
        }
    }
    Ok(fresh)
}

// needed helper: merges one notification into the roster, tracking unseen peers
fn absorb_update(
    roster: &mut discovery::link::RosterClient,
    update: freenet_stdlib::prelude::UpdateData<'static>,
    fresh: &mut Vec<discovery::membership::PeerEntry>,
) {
    use freenet_stdlib::prelude::UpdateData;
    let bytes = match update {
        UpdateData::State(s) => Some(s.as_ref().to_vec()),
        UpdateData::Delta(d) => Some(d.as_ref().to_vec()),
        UpdateData::StateAndDelta { state, .. } => Some(state.as_ref().to_vec()),
        _ => None,
    };
    if let Some(bytes) = bytes {
        absorb_bytes(roster, &bytes, fresh);
    }
}

// needed helper: merges raw roster bytes, tracking unseen or changed peers
fn absorb_bytes(
    roster: &mut discovery::link::RosterClient,
    bytes: &[u8],
    fresh: &mut Vec<discovery::membership::PeerEntry>,
) {
    let incoming: discovery::membership::RosterState =
        bincode::deserialize(bytes).unwrap_or_default();
    for (id, entry) in &incoming {
        if **id == *roster.own {
            continue;
        }
        let unseen = !roster.slots.contains_key(id);
        let changed = roster
            .slots
            .get(id)
            .is_some_and(|known| known.peer_id != entry.peer_id);
        if unseen || changed {
            fresh.push(entry.clone());
        }
    }
    roster.slots = discovery::membership::merge_roster(roster.slots.clone(), incoming);
    note_foreign(roster);
}

// needed helper: refreshes the foreign-activity baseline from entry timestamps
fn note_foreign(roster: &mut discovery::link::RosterClient) {
    let own = roster.own;
    let sum: u64 = roster
        .slots
        .iter()
        .filter(|(id, _)| **id != own)
        .map(|(_, e)| *e.updated_at)
        .sum();
    if sum != roster.foreign_sum {
        roster.foreign_sum = sum;
        roster.foreign_seen = Some(std::time::Instant::now());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        std::hint::black_box(super::poll);
    }
}
