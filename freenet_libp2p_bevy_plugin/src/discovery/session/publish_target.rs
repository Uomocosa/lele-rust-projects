use crate::discovery;
use discovery::session::{PublishTarget, Session};

#[must_use]
pub fn publish_target(session: &Session) -> Option<PublishTarget> {
    session.room.as_ref().map(|room| PublishTarget {
        room: room.name.clone(),
        addrs: session.addrs.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::publish_target;
    use crate::discovery;
    use crate::net_id;
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            net_id::PeerId::from("me"),
            vec![net_id::PeerAddr::from("/ip4/9")],
            discovery::Timing::default(),
        );
        assert!(publish_target(&session).is_none());
        session.room = Some(discovery::Room {
            name: net_id::RoomName::from("r"),
            members: discovery::Members::new(),
        });
        let target = publish_target(&session);
        assert_eq!(
            target.map(|target| target.addrs),
            Some(vec![net_id::PeerAddr::from("/ip4/9")])
        );
    }
}
