use crate::discovery;
use crate::p2p;
use discovery::id::RemotePeerId;
use discovery::link::NetLink;
use discovery::session::{Session, hello_for};

pub fn send_hello(session: &Session, link: &NetLink, peer: &RemotePeerId) {
    let data = bincode::serialize(&hello_for(session)).unwrap_or_default();
    link.tx
        .send(p2p::NetCommand::Exchange {
            peer_id: (**peer).clone(),
            data,
        })
        .ok();
}

#[cfg(test)]
mod tests {
    use super::send_hello;
    use crate::discovery;
    use crate::p2p;
    use discovery::id::RemotePeerId;

    #[test]
    fn test_usage() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (_, observed) = tokio::sync::watch::channel(None);
        let link = discovery::link::NetLink { tx, observed };
        let session = discovery::session::Session::new(
            discovery::id::UniqueGameId::new(
                &discovery::id::GameName("g".to_string()),
                &discovery::id::GameToken("t".to_string()),
            ),
            RemotePeerId("me".to_string()),
            vec!["/ip4/9".to_string()],
            8,
        );
        send_hello(&session, &link, &RemotePeerId("a".to_string()));
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::NetCommand::Exchange { peer_id, .. }) if peer_id == "a"
        ));
    }
}
