use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use crate::discovery;
use crate::discovery::Discovery;
use crate::discovery_update_data_bytes;
use crate::frame::Frame;
use crate::frame_verify_frame;

pub async fn poll(d: &mut Discovery) {
    while let Some(Ok(resp)) = d.client.recv_with_timeout(Duration::from_millis(10)).await {
        if let HostResponse::ContractResponse(
            freenet_stdlib::client_api::ContractResponse::UpdateNotification { update, .. },
        ) = resp
            && let Some(bytes) = discovery_update_data_bytes::update_data_bytes(&update)
            && let Ok(data) = bincode::deserialize::<discovery::StateData>(&bytes)
        {
            for (k, v) in data.peers {
                d.peers.entry(k).or_insert(v);
            }
            for (seq, e) in data.chain {
                let frame = Frame {
                    seq,
                    prev: e.prev,
                    next: e.next,
                    author: e.author,
                    sig: e.sig.clone(),
                };
                if frame_verify_frame::verify_frame(&frame) {
                    d.chain.entry(seq).or_insert(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(poll);
    }
}
