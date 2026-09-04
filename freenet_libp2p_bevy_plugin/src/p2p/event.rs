use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event<T> {
    Ready {
        peer_id: String,
        addrs: Vec<String>,
    },
    PeerConnected(String),
    PeerDisconnected(String),
    Message {
        from: String,
        payload: T,
    },
    HistoryChunk {
        lobby: String,
        chunk: u64,
        data: Vec<u8>,
    },
    Error(String),
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::Event;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let e = Event::<Dummy>::Ready {
            peer_id: "p".to_string(),
            addrs: vec![],
        };
        assert!(matches!(e, Event::Ready { .. }));
    }
}
