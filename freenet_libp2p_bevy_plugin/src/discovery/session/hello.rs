use crate::discovery;
use discovery::session::{Hello, Session};

#[must_use]
pub fn hello(session: &Session) -> Hello {
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
    use super::hello;
    use crate::discovery;
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let session = Session::new(
            discovery::PeerId("me".to_string()),
            vec!["/ip4/9".to_string()],
            discovery::Timing::default(),
        );
        let hello = hello(&session);
        assert_eq!(hello.room, None);
        assert_eq!(hello.addrs, vec!["/ip4/9".to_string()]);
    }
}
