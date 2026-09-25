use crate::p2p;

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::super::constants;
use super::super::dial::rank_addrs::rank_addrs;
use super::super::membership::peer_entry::PeerEntry;
use super::dial_addrs::dial_addrs;
use super::maps::{ConnectedMap, StaggerMap};
use super::with_loopback::with_loopback;

pub fn dial_preferred_raw(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    connected: &ConnectedMap,
    staggers: &mut StaggerMap,
    mode: p2p::TransportMode,
    entry: &PeerEntry,
) {
    if connected.contains_key(&entry.peer_id) {
        staggers.remove(&entry.peer_id);
        return;
    }
    let ranked = rank_addrs(&with_loopback(&entry.addrs), mode);
    let mut queue: VecDeque<String> = ranked.into();
    let Some(first) = queue.pop_front() else {
        return;
    };
    dial_addrs(net_tx, &entry.peer_id, &[first], &entry.addrs);
    if queue.is_empty() {
        staggers.remove(&entry.peer_id);
        return;
    }
    let due = Instant::now()
        .checked_add(Duration::from_secs(constants::STAGGER_SECS))
        .unwrap_or_else(Instant::now);
    staggers.insert(entry.peer_id.clone(), (queue, due));
}

// no test_usage necessary
