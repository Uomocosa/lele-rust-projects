use std::collections::BTreeMap;

use super::super::id::room_name::RoomName;
use super::super::id::room_record::RoomRecord;

pub type RoomCatalogue = BTreeMap<RoomName, RoomRecord>;
// no test_usage necessary
