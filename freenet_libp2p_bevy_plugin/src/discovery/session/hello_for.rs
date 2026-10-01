use crate::discovery;
use discovery::session::{Hello, Session};

#[must_use]
pub fn hello_for(session: &Session) -> Hello {
    let Some(room) = &session.room else {
        return Hello {
            room: None,
            addrs: session.addrs.clone(),
            peers: Vec::new(),
        };
    };
    let peers = room
        .members
        .iter()
        .map(|(peer, member)| (peer.clone(), member.presence.addrs.clone()))
        .collect();
    Hello {
        room: Some(room.name.clone()),
        addrs: session.addrs.clone(),
        peers,
    }
}

#[cfg(test)]
mod tests {
    use super::hello_for;
    use crate::discovery;
    use discovery::id::RemotePeerId;

    #[test]
    fn test_usage() {
        let session = discovery::session::Session::new(
            discovery::id::UniqueGameId::new(
                &discovery::id::GameName("g".to_string()),
                &discovery::id::GameToken("t".to_string()),
            ),
            RemotePeerId("me".to_string()),
            vec!["/ip4/9".to_string()],
            8,
        );
        let hello = hello_for(&session);
        assert_eq!(hello.room, None);
        assert_eq!(hello.addrs, vec!["/ip4/9".to_string()]);
    }
}
