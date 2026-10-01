use crate::discovery;
use discovery::link::NetLink;
use discovery::session::{Session, send_hello};

pub fn broadcast_hello(session: &Session, link: &NetLink) {
    for peer in &session.connected {
        send_hello(session, link, peer);
    }
}

#[cfg(test)]
mod tests {
    use super::broadcast_hello;
    use crate::discovery;
    use crate::p2p;
    use discovery::id::RemotePeerId;

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (_, observed) = tokio::sync::watch::channel(None);
        let link = discovery::link::NetLink { tx, observed };
        let mut session = discovery::session::Session::new(
            discovery::id::UniqueGameId::new(
                &discovery::id::GameName("g".to_string()),
                &discovery::id::GameToken("t".to_string()),
            ),
            RemotePeerId("me".to_string()),
            vec!["/ip4/9".to_string()],
            8,
        );
        session.connected.insert(RemotePeerId("a".to_string()));
        session.connected.insert(RemotePeerId("b".to_string()));
        broadcast_hello(&session, &link);
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::NetCommand::Exchange { .. })
        ));
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::NetCommand::Exchange { .. })
        ));
    }
}
