use std::collections::{BTreeMap, BTreeSet};

use crate::discovery;
use crate::net_id;
use discovery::session::Output;

pub struct Session {
    pub me: net_id::PeerId,
    pub addrs: Vec<net_id::PeerAddr>,
    pub timing: discovery::Timing,
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::Room>,
    pub connected: BTreeSet<net_id::PeerId>,
    pub candidates: BTreeMap<net_id::PeerId, discovery::Presence>,
    pub last_dial: BTreeMap<net_id::PeerId, discovery::EpochSecs>,
    pub last_hello: Option<discovery::EpochSecs>,
    pub outputs: Vec<Output>,
}

impl Session {
    #[must_use]
    pub const fn new(
        me: net_id::PeerId,
        addrs: Vec<net_id::PeerAddr>,
        timing: discovery::Timing,
    ) -> Self {
        Self {
            me,
            addrs,
            timing,
            lobby: discovery::Lobby::new(),
            room: None,
            connected: BTreeSet::new(),
            candidates: BTreeMap::new(),
            last_dial: BTreeMap::new(),
            last_hello: None,
            outputs: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Session;
    use crate::discovery;
    use crate::net_id;

    #[test]
    fn test_usage() {
        let session = Session::new(
            net_id::PeerId::from("me"),
            Vec::new(),
            discovery::Timing::default(),
        );
        assert!(session.room.is_none());
        assert_eq!(session.outputs, Vec::new());
    }
}
