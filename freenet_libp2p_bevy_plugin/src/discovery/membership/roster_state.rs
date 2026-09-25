use std::collections::BTreeMap;

use super::super::params::player_id::PlayerId;
use super::peer_entry::PeerEntry;

pub type RosterState = BTreeMap<PlayerId, PeerEntry>;
// no test_usage necessary
