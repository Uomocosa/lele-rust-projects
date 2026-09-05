use freenet_stdlib::client_api::{ClientRequest, ContractRequest};

use crate::discovery;
use crate::discovery::Discovery;
use crate::frame::Frame;

/// # Errors
/// Returns error if send fails.
///
/// # Panics
/// May panic if serialization fails.
pub async fn publish_frame(d: &mut Discovery, frame: &Frame) -> Result<(), String> {
    let mut delta = discovery::StateData::default();
    delta.chain.insert(
        frame.seq,
        discovery::ChainEntry {
            author: frame.author,
            prev: frame.prev,
            next: frame.next,
            sig: frame.sig.clone(),
        },
    );
    let serialized = bincode::serialize(&delta).unwrap_or_default();
    let data = freenet_stdlib::prelude::State::from(serialized);
    let req = ClientRequest::ContractOp(ContractRequest::Update {
        key: d.key,
        data: freenet_stdlib::prelude::UpdateData::Delta(
            freenet_stdlib::prelude::StateDelta::from(data.as_ref().to_vec()),
        ),
    });
    d.client.send(req).await?;
    d.chain.insert(
        frame.seq,
        discovery::ChainEntry {
            author: frame.author,
            prev: frame.prev,
            next: frame.next,
            sig: frame.sig.clone(),
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(publish_frame);
    }
}
