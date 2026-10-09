use std::collections::BTreeMap;

use crate::discovery;
use crate::net_id;

pub type Lobby = BTreeMap<net_id::RoomName, discovery::RoomRecord>;
pub type Members = BTreeMap<net_id::PeerId, discovery::Member>;
