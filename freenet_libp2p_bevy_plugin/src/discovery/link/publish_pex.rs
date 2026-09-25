use crate::p2p;

use super::super::params::unique_game_id::UniqueGameId;
use super::pex_msg::PexMsg;
use super::pex_topic::pex_topic;

pub fn publish_pex(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    id: &UniqueGameId,
    msg: &PexMsg,
) {
    let data = bincode::serialize(msg).unwrap_or_default();
    net_tx
        .send(p2p::NetCommand::Publish {
            topic: pex_topic(id),
            data,
        })
        .ok();
}

// no test_usage necessary
