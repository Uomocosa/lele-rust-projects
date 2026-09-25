use crate::p2p;

use super::super::params::room_name::RoomName;
use super::super::params::unique_game_id::UniqueGameId;
use super::pex_topic::pex_topic;
use super::roster_topic::roster_topic;

pub fn subscribe_topics(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    id: &UniqueGameId,
    room: &RoomName,
) {
    for topic in [pex_topic(id), roster_topic(id, room)] {
        net_tx.send(p2p::NetCommand::Subscribe { topic }).ok();
    }
}

// no test_usage necessary
