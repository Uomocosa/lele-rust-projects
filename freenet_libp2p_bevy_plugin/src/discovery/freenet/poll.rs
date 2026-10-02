use std::time::Duration;

use freenet_stdlib::client_api::{ContractResponse, HostResponse};
use freenet_stdlib::prelude::UpdateData;

use crate::discovery;

const DRAIN_TIMEOUT: Duration = Duration::from_millis(10);

pub async fn poll(
    directory_client: &mut discovery::freenet::DirectoryClient,
) -> Result<discovery::Directory, discovery::Error> {
    while let Some(result) = directory_client
        .client
        .recv_response_timeout(DRAIN_TIMEOUT)
        .await
    {
        match result? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification {
                update, ..
            }) => absorb_update(directory_client, update),
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                absorb_bytes(directory_client, state.as_ref());
            }
            _ => {}
        }
    }
    Ok(directory_client.directory.clone())
}

// needed helper: merges one notification into the cached directory
fn absorb_update(
    directory_client: &mut discovery::freenet::DirectoryClient,
    update: UpdateData<'static>,
) {
    let bytes = match update {
        UpdateData::State(state) | UpdateData::StateAndDelta { state, .. } => {
            Some(state.as_ref().to_vec())
        }
        UpdateData::Delta(delta) => Some(delta.as_ref().to_vec()),
        _ => None,
    };
    if let Some(bytes) = bytes {
        absorb_bytes(directory_client, &bytes);
    }
}

// needed helper: merges raw directory bytes into the cached directory
fn absorb_bytes(directory_client: &mut discovery::freenet::DirectoryClient, bytes: &[u8]) {
    let incoming: discovery::Directory = bincode::deserialize(bytes).unwrap_or_default();
    let cached = std::mem::take(&mut directory_client.directory);
    directory_client.directory = discovery::freenet::merge_directory(cached, incoming);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        std::hint::black_box(super::poll);
    }
}
