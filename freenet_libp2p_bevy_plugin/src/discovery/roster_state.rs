use std::collections::BTreeMap;

use super::peer_entry::PeerEntry;
use super::player_id::PlayerId;

pub type RosterState = BTreeMap<PlayerId, PeerEntry>;
// no test_usage necessary
