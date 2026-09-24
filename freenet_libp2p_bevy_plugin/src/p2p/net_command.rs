use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetCommand {
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
    PutHistory {
        lobby: String,
        chunk: u64,
        data: Vec<u8>,
    },
    FetchHistory {
        lobby: String,
        chunk: u64,
    },
    FetchRoster {
        lobby: String,
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
    use super::NetCommand;

    #[test]
    fn test_usage() {
        let command = NetCommand::Dial {
            peer_id: "p".to_string(),
            addrs: vec![],
        };
        assert!(matches!(command, NetCommand::Dial { .. }));
    }
}
