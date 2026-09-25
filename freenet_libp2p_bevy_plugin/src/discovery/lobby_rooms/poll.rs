use std::time::Duration;

use freenet_stdlib::client_api::{ContractResponse, HostResponse};
use freenet_stdlib::prelude::UpdateData;

use super::super::error::Error;
use super::catalogue::RoomCatalogue;
use super::index_client::IndexClient;
use super::merge_board::merge_board;

const DRAIN_TIMEOUT: Duration = Duration::from_millis(10);

pub async fn poll(client: &mut IndexClient) -> Result<RoomCatalogue, Error> {
    let mut board: Option<RoomCatalogue> = None;
    while let Some(result) = client.client.recv_response_timeout(DRAIN_TIMEOUT).await {
        match result? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification {
                update, ..
            }) => absorb_update(update, &mut board),
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                absorb_bytes(state.as_ref(), &mut board);
            }
            _ => {}
        }
    }
    board.ok_or(Error::Disconnected)
}

// needed helper: merges one notification into the latest board snapshot
fn absorb_update(update: UpdateData<'static>, board: &mut Option<RoomCatalogue>) {
    let bytes = match update {
        UpdateData::State(state) | UpdateData::StateAndDelta { state, .. } => {
            Some(state.as_ref().to_vec())
        }
        UpdateData::Delta(delta) => Some(delta.as_ref().to_vec()),
        _ => None,
    };
    if let Some(bytes) = bytes {
        let incoming: RoomCatalogue = bincode::deserialize(&bytes).unwrap_or_default();
        merge_into(board, incoming);
    }
}

// needed helper: merges raw board bytes into the latest snapshot
fn absorb_bytes(bytes: &[u8], board: &mut Option<RoomCatalogue>) {
    let incoming: RoomCatalogue = bincode::deserialize(bytes).unwrap_or_default();
    merge_into(board, incoming);
}

// needed helper: last snapshot wins, merged by freshness
fn merge_into(board: &mut Option<RoomCatalogue>, incoming: RoomCatalogue) {
    match board {
        Some(current) => *current = merge_board(current.clone(), incoming),
        None => *board = Some(incoming),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        std::hint::black_box(super::poll);
    }
}
