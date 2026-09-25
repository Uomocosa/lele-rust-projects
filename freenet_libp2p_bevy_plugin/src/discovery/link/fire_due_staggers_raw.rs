use crate::p2p;

use std::time::Instant;

use super::super::dial::stagger_due::stagger_due;
use super::maps::{ConnectedMap, StaggerMap};

pub fn fire_due_staggers_raw(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
) {
    for (peer, addr) in stagger_due(Instant::now(), connected, staggers) {
        net_tx
            .send(p2p::NetCommand::DialForce {
                peer_id: peer,
                addrs: vec![addr],
            })
            .ok();
    }
}

// no test_usage necessary
