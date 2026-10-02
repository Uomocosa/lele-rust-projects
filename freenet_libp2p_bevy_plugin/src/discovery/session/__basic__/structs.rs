use serde::{Deserialize, Serialize};

use crate::discovery;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Hello {
    pub room: Option<discovery::RoomName>,
    pub addrs: Vec<String>,
    pub peers: Vec<(discovery::PeerId, Vec<String>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishTarget {
    pub room: discovery::RoomName,
    pub addrs: Vec<String>,
}
