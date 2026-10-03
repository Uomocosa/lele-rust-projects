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
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let session = Session::new(
            discovery::PeerId("me".to_string()),
            vec!["/ip4/9".to_string()],
            discovery::Timing::default(),
        );
        assert_eq!(snapshot(&session), discovery::Snapshot::default());
    }
}
