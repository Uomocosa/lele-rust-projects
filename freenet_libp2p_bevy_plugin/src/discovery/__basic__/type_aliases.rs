use std::collections::BTreeMap;

use crate::discovery;

pub type Directory = BTreeMap<discovery::RoomName, discovery::RoomRecord>;
pub type Members = BTreeMap<discovery::PeerId, discovery::Member>;
