use crate::discovery;
use discovery::session::Session;

pub fn broadcast_hello(session: &mut Session) {
    let peers: Vec<discovery::PeerId> = session.connected.iter().cloned().collect();
    for peer in &peers {
        discovery::session::send_hello(session, peer);
    }
}

#[cfg(test)]
mod tests {
    use super::broadcast_hello;
    use crate::discovery;
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            discovery::PeerId("me".to_string()),
            vec!["/ip4/9".to_string()],
            discovery::Timing::default(),
        );
        session.connected.insert(discovery::PeerId("a".to_string()));
        session.connected.insert(discovery::PeerId("b".to_string()));
        broadcast_hello(&mut session);
        assert_eq!(std::mem::take(&mut session.outputs).len(), 2);
    }
}
