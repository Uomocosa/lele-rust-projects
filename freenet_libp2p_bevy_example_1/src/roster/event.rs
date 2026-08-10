use std::collections::BTreeMap;

use crate::boxes;
use crate::roster;

#[derive(Debug, Clone)]
pub enum Event {
    Roster {
        entries: BTreeMap<boxes::PlayerId, roster::PeerEntry>,
    },
    ConnectionError(String),
}
