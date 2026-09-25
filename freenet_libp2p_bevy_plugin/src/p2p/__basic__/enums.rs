use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    Tcp,
    Quic,
    Both,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event<T> {
    Ready {
        peer_id: String,
        addrs: Vec<String>,
    },
    PeerConnected(String),
    PeerDisconnected(String),
    DialFailed {
        peer_id: String,
        reason: String,
    },
    RelayReserved {
        relay_peer_id: String,
    },
    ObservedAddr(String),
    LobbyProviders {
        lobby: String,
        peers: Vec<String>,
    },
    Message {
        from: String,
        payload: T,
    },
    HistoryChunk {
        lobby: String,
        chunk: u64,
        data: Vec<u8>,
    },
    Gossip {
        topic: String,
        from: String,
        data: Vec<u8>,
    },
    Error(String),
}
