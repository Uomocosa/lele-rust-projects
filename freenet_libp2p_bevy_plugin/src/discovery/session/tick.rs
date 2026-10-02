use crate::discovery;
use discovery::session::Session;

pub fn tick(session: &mut Session, now: discovery::EpochSecs) {
    discovery::session::dial_candidates(session, now);
    let hello_due = session
        .last_hello
        .is_none_or(|last| now.saturating_sub(*last) >= session.timing.hello_secs);
    if hello_due && session.room.is_some() {
        discovery::session::broadcast_hello(session);
        session.last_hello = Some(now);
    }
    discovery::session::prune_members(session, now);
}

#[cfg(test)]
mod tests {
    use super::tick;
    use crate::discovery;
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            discovery::PeerId("me".to_string()),
            Vec::new(),
            discovery::Timing::default(),
        );
        session.connected.insert(discovery::PeerId("a".to_string()));
        session.room = Some(discovery::Room {
            name: discovery::RoomName("r".to_string()),
            members: discovery::Members::new(),
        });
        tick(&mut session, discovery::EpochSecs(100));
        tick(&mut session, discovery::EpochSecs(101));
        assert_eq!(session.last_hello, Some(discovery::EpochSecs(100)));
        assert_eq!(std::mem::take(&mut session.outputs).len(), 1);
    }
}
