use crate::p2p;

use super::pex_msg::PexMsg;
use super::pex_topic::pex_topic;

pub fn publish_pex(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    namespace: &str,
    msg: &PexMsg,
) {
    let data = bincode::serialize(msg).unwrap_or_default();
    net_tx
        .send(p2p::NetCommand::Publish {
            topic: pex_topic(namespace),
            data,
        })
        .ok();
}

// no test_usage necessary
