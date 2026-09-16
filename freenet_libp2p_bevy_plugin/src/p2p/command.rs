use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command<T> {
    Dial {
        peer_id: String,
        addrs: Vec<String>,
    },
    DialForce {
        peer_id: String,
        addrs: Vec<String>,
    },
    ReserveRelay {
        relay_addr: String,
    },
    SetMdns {
        enabled: bool,
    },
    AddKadPeer {
        peer_id: String,
        addrs: Vec<String>,
    },
    ProvideLobby {
        lobby: String,
    },
    FindLobby {
        lobby: String,
    },
    Send {
        peer_id: String,
        payload: T,
    },
    PutHistory {
        lobby: String,
        chunk: u64,
        data: Vec<u8>,
    },
    FetchHistory {
        lobby: String,
        chunk: u64,
    },
    Subscribe {
        topic: String,
    },
    Publish {
        topic: String,
        data: Vec<u8>,
    },
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::Command;
    use derive_more::Deref;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let c = Command::Dial {
            peer_id: "p".to_string(),
            addrs: vec![],
        };
        let _: Command<Dummy> = c;
    }
}
