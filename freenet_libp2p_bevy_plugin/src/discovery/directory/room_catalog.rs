use std::collections::BTreeMap;

use super::super::params::room_name::RoomName;
use super::room_payload::RoomPayload;

pub type RoomCatalog = BTreeMap<RoomName, RoomPayload>;
// no test_usage necessary
