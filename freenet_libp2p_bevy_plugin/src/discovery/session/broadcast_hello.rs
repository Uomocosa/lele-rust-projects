use crate::discovery;
use crate::net_id;
use discovery::session::Session;

pub fn broadcast_hello(session: &mut Session) {
    let peers: Vec<net_id::PeerId> = session.connected.iter().cloned().collect();
    for peer in &peers {
        discovery::session::send_hello(session, peer);
    }
}

#[cfg(test)]
mod tests {
    use super::broadcast_hello;
    use crate::discovery;
    use crate::net_id;
    use discovery::session::Session;

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            net_id::PeerId("me".to_string()),
            vec![net_id::PeerAddr("/ip4/9".to_string())],
            discovery::Timing::default(),
        );
        session.connected.insert(net_id::PeerId("a".to_string()));
        session.connected.insert(net_id::PeerId("b".to_string()));
        broadcast_hello(&mut session);
        assert_eq!(std::mem::take(&mut session.outputs).len(), 2);
    }
}
