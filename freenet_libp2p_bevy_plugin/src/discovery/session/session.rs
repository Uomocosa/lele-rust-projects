use std::collections::{BTreeMap, BTreeSet};

use crate::discovery;
use discovery::session::Output;

pub struct Session {
    pub me: discovery::PeerId,
    pub addrs: Vec<String>,
    pub timing: discovery::Timing,
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::Room>,
    pub connected: BTreeSet<discovery::PeerId>,
    pub candidates: BTreeMap<discovery::PeerId, discovery::Presence>,
    pub last_dial: BTreeMap<discovery::PeerId, discovery::EpochSecs>,
    pub last_hello: Option<discovery::EpochSecs>,
    pub outputs: Vec<Output>,
}

impl Session {
    #[must_use]
    pub const fn new(me: discovery::PeerId, addrs: Vec<String>, timing: discovery::Timing) -> Self {
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

    #[test]
    fn test_usage() {
        let session = Session::new(
            discovery::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        assert!(session.room.is_none());
        assert_eq!(session.outputs, Vec::new());
    }
}
