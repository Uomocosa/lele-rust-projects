use serde::{Deserialize, Serialize};

use super::net_command::NetCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command<T> {
    Net(NetCommand),
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
    use serde::{Deserialize, Serialize};

    use super::Command;
    use crate::p2p;
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
        let net: Command<Dummy> = Command::Net(p2p::NetCommand::FindLobby {
            lobby: "l".to_string(),
        });
        assert!(matches!(net, Command::Net(_)));
    }
}
