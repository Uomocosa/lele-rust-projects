use std::collections::BTreeMap;

use super::newtypes::{RemotePeerId, RoomName};
use super::structs::{Member, RoomRecord};

pub type RoomCatalogue = BTreeMap<RoomName, RoomRecord>;
pub type Members = BTreeMap<RemotePeerId, Member>;
// no test_usage necessary
