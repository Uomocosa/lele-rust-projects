use crate::p2p;

use super::super::membership::roster_state::RosterState;
use super::super::params::player_id::PlayerId;
use super::dial_preferred_raw::dial_preferred_raw;
use super::maps::{ConnectedMap, StaggerMap};

pub fn dial_known(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    slots: &RosterState,
    own: PlayerId,
    mode: p2p::TransportMode,
) {
    for (id, entry) in slots {
        if *id == own {
            continue;
        }
        dial_preferred_raw(net_tx, connected, staggers, mode, entry);
    }
}

// no test_usage necessary
