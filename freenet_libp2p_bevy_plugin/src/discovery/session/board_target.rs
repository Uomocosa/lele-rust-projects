use crate::discovery;
use discovery::session::{BoardTarget, Session};

#[must_use]
pub fn board_target(session: &Session) -> Option<BoardTarget> {
    session.room.as_ref().map(|room| BoardTarget {
        room: room.name.clone(),
        addrs: session.addrs.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::board_target;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let session = discovery::session::Session::new(
            discovery::id::UniqueGameId::new(
                &discovery::id::GameName("g".to_string()),
                &discovery::id::GameToken("t".to_string()),
            ),
            discovery::id::RemotePeerId("me".to_string()),
            Vec::new(),
            8,
        );
        assert!(board_target(&session).is_none());
    }
}
