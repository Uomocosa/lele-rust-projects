use std::collections::BTreeMap;

use crate::discovery::basic::newtypes::{RemotePeerId, RoomName};
use crate::discovery::basic::structs::{Member, RoomRecord};

pub type RoomCatalogue = BTreeMap<RoomName, RoomRecord>;
pub type Members = BTreeMap<RemotePeerId, Member>;
// no test_usage necessary
