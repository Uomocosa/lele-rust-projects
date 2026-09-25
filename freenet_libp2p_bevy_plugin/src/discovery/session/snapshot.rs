use super::super::Multiplayer;
use super::session::Session;

#[must_use]
pub fn snapshot(session: &Session) -> Multiplayer {
    Multiplayer {
        catalogue: session.catalogue.clone(),
        room: session.room.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::snapshot;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let session = discovery::session::Session::new(
            discovery::id::UniqueGameId::new(
                &discovery::id::GameName("test".to_string()),
                &discovery::id::GameToken("token".to_string()),
            ),
            discovery::id::RemotePeerId("me".to_string()),
            Vec::new(),
            8,
        );
        let snapshot = snapshot(&session);
        assert!(snapshot.room.is_none());
    }
}
