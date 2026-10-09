use crate::discovery;
use crate::net_id;
use crate::p2p;
use discovery::session::{Output, Session};

pub fn send_hello(session: &mut Session, peer: &net_id::PeerId) {
    let data = bincode::serialize(&discovery::session::hello(session)).unwrap_or_default();
    session.outputs.push(Output::Net(p2p::NetCommand::Exchange {
        peer_id: peer.clone(),
        data,
    }));
}

#[cfg(test)]
mod tests {
    use super::send_hello;
    use crate::discovery;
    use crate::net_id;
    use crate::p2p;
    use discovery::session::{Output, Session};

    #[test]
    fn test_usage() {
        let mut session = Session::new(
            net_id::PeerId("me".to_string()),
            vec![net_id::PeerAddr("/ip4/9".to_string())],
            discovery::Timing::default(),
        );
        send_hello(&mut session, &net_id::PeerId("a".to_string()));
        assert!(matches!(
            std::mem::take(&mut session.outputs).as_slice(),
            [Output::Net(p2p::NetCommand::Exchange { peer_id, .. })] if peer_id.as_str() == "a"
        ));
    }
}
