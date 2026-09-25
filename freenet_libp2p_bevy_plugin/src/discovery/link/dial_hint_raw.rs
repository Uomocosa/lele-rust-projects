use crate::p2p;

use super::super::constants;
use super::super::dial::decide_dial::decide_dial;
use super::super::dial::decision::Decision;
use super::super::gossip::peer_hint::PeerHint;
use super::super::membership::peer_entry::PeerEntry;
use super::dial_preferred_raw::dial_preferred_raw;
use super::epoch_secs::epoch_secs;
use super::maps::{AttemptedMap, ConnectedMap, StaggerMap};
use super::with_loopback::with_loopback;
use std::time::Instant;

pub fn dial_hint_raw(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    attempted: &mut AttemptedMap,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    peer_id: &str,
    mode: p2p::TransportMode,
    hint: &PeerHint,
) {
    if hint.peer_id.is_empty() || hint.peer_id == peer_id || hint.addrs.is_empty() {
        return;
    }
    if epoch_secs().saturating_sub(hint.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    let entry = PeerEntry {
        peer_id: hint.peer_id.clone(),
        addrs: hint.addrs.clone(),
        updated_at: hint.updated_at,
    };
    let age = attempted.get(&entry.peer_id).and_then(|seen| {
        Instant::now()
            .checked_duration_since(*seen)
            .map(|d| d.as_secs())
    });
    if age.is_some_and(|seen_secs| seen_secs < constants::REDIAL_SECS) {
        return;
    }
    match decide_dial(peer_id, &entry.peer_id, age) {
        Decision::Wait => {
            attempted
                .entry(entry.peer_id.clone())
                .or_insert_with(Instant::now);
        }
        Decision::Dial => {
            attempted.insert(entry.peer_id.clone(), Instant::now());
            dial_preferred_raw(net_tx, connected, staggers, mode, &entry);
        }
        Decision::ForceDial => {
            attempted.insert(entry.peer_id.clone(), Instant::now());
            net_tx
                .send(p2p::NetCommand::DialForce {
                    peer_id: entry.peer_id.clone(),
                    addrs: with_loopback(&entry.addrs),
                })
                .ok();
        }
    }
}

// no test_usage necessary
