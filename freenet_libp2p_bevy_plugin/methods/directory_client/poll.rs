use std::time::Duration;

use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::discovery;

const DRAIN_TIMEOUT: Duration = Duration::from_millis(10);

pub async fn poll(
    directory: &mut discovery::link::DirectoryClient,
) -> Result<discovery::directory::DirectoryState, discovery::Error> {
    while let Some(result) = directory.client.recv_response_timeout(DRAIN_TIMEOUT).await {
        match result? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification {
                update, ..
            }) => absorb_update(directory, update),
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                absorb_bytes(directory, state.as_ref());
            }
            _ => {}
        }
    }
    Ok(directory.slots.clone())
}

// needed helper: merges one notification into the directory
fn absorb_update(
    directory: &mut discovery::link::DirectoryClient,
    update: freenet_stdlib::prelude::UpdateData<'static>,
) {
    use freenet_stdlib::prelude::UpdateData;
    let bytes = match update {
        UpdateData::State(s) => Some(s.as_ref().to_vec()),
        UpdateData::Delta(d) => Some(d.as_ref().to_vec()),
        UpdateData::StateAndDelta { state, .. } => Some(state.as_ref().to_vec()),
        _ => None,
    };
    if let Some(bytes) = bytes {
        absorb_bytes(directory, &bytes);
    }
}

// needed helper: merges raw directory bytes
fn absorb_bytes(directory: &mut discovery::link::DirectoryClient, bytes: &[u8]) {
    let incoming: discovery::directory::DirectoryState =
        bincode::deserialize(bytes).unwrap_or_default();
    directory.slots = discovery::directory::merge_directory(directory.slots.clone(), incoming);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        std::hint::black_box(super::poll);
    }
}
