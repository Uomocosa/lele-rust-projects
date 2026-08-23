use std::collections::BTreeMap;

use crate::net_id;
use crate::roster;

#[derive(Debug, Clone)]
pub enum Event {
    Connecting {
        attempt: u32,
    },
    Roster {
        entries: BTreeMap<net_id::NetworkId, roster::PeerEntry>,
    },
    ConnectionError(String),
}
