use std::time::Duration;

use freenet_stdlib::client_api::{ContractResponse, HostResponse};
use freenet_stdlib::prelude::UpdateData;

use crate::discovery;
use discovery::Error;
use discovery::lobby_rooms::merge_board::merge_board;
use discovery::lobby_rooms::{IndexClient, RoomCatalogue};

const DRAIN_TIMEOUT: Duration = Duration::from_millis(10);

pub async fn poll(client: &mut IndexClient) -> Result<RoomCatalogue, Error> {
    while let Some(result) = client.client.recv_response_timeout(DRAIN_TIMEOUT).await {
        match result? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification {
                update, ..
            }) => absorb_update(client, update),
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                absorb_bytes(client, state.as_ref());
            }
            _ => {}
        }
    }
    Ok(client.slots.clone())
}

// needed helper: merges one notification into the cached board
fn absorb_update(client: &mut IndexClient, update: UpdateData<'static>) {
    let bytes = match update {
        UpdateData::State(state) | UpdateData::StateAndDelta { state, .. } => {
            Some(state.as_ref().to_vec())
        }
        UpdateData::Delta(delta) => Some(delta.as_ref().to_vec()),
        _ => None,
    };
    if let Some(bytes) = bytes {
        absorb_bytes(client, &bytes);
    }
}

// needed helper: merges raw board bytes into the cached board
fn absorb_bytes(client: &mut IndexClient, bytes: &[u8]) {
    let incoming: RoomCatalogue = bincode::deserialize(bytes).unwrap_or_default();
    client.slots = merge_board(client.slots.clone(), incoming);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        std::hint::black_box(super::poll);
    }
}
