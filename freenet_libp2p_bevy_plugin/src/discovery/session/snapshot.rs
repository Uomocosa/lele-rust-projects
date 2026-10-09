use crate::discovery;
use discovery::session::Session;

#[must_use]
pub fn snapshot(session: &Session) -> discovery::Snapshot {
    discovery::Snapshot {
        lobby: session.lobby.clone(),
        room: session.room.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::snapshot;
    use crate::discovery;
    use crate::net_id;
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let session = Session::new(
            net_id::PeerId::from("me"),
            vec![net_id::PeerAddr::from("/ip4/9")],
            discovery::Timing::default(),
        );
        assert_eq!(snapshot(&session), discovery::Snapshot::default());
    }
}
