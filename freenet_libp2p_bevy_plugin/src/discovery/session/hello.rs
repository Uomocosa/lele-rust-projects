use crate::discovery;
use crate::net_id;
use discovery::session::{Hello, Session};

#[must_use]
pub fn hello(session: &Session) -> Hello {
    let Some(room) = &session.room else {
        return Hello {
            room: None,
            addrs: session.me.addrs.clone(),
            peers: Vec::new(),
        };
    };
    let peers = room
        .members
        .iter()
        .map(|(peer, member)| net_id::Peer {
            id: peer.clone(),
            addrs: member.presence.addrs.clone(),
        })
        .collect();
    Hello {
        room: Some(room.name.clone()),
        addrs: session.me.addrs.clone(),
        peers,
    }
}

#[cfg(test)]
mod tests {
    use super::hello;
    use crate::discovery;
    use crate::net_id;
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let session = Session::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: vec![net_id::PeerAddr::from("/ip4/9")],
            },
            discovery::Timing::default(),
        );
        let hello = hello(&session);
        assert_eq!(hello.room, None);
        assert_eq!(hello.addrs, vec![net_id::PeerAddr::from("/ip4/9")]);
    }
}
