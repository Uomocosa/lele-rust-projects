use crate::p2p;

use super::pex_topic::pex_topic;
use super::roster_topic::roster_topic;

pub fn subscribe_topics(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    namespace: &str,
    room: &str,
) {
    for topic in [pex_topic(namespace), roster_topic(namespace, room)] {
        net_tx.send(p2p::NetCommand::Subscribe { topic }).ok();
    }
}

// no test_usage necessary
