#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TapEvent {
    Ready {
        peer_id: String,
        addrs: Vec<String>,
    },
    ObservedAddr(String),
    PeerConnected(String),
    PeerDisconnected(String),
    DialFailed {
        peer_id: String,
        reason: String,
    },
    LobbyProviders {
        lobby: String,
        peers: Vec<String>,
    },
    Gossip {
        topic: String,
        from: String,
        data: Vec<u8>,
    },
}

#[cfg(test)]
mod tests {
    use super::TapEvent;

    #[test]
    fn test_usage() {
        let event = TapEvent::PeerConnected("peer".to_string());
        assert!(matches!(event, TapEvent::PeerConnected(_)));
        let gossip = TapEvent::Gossip {
            topic: "t".to_string(),
            from: "p".to_string(),
            data: vec![1, 2],
        };
        assert!(matches!(gossip, TapEvent::Gossip { .. }));
    }
}
