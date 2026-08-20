use crate::boxes;
use crate::roster;

pub struct ConnectAndRunArgs {
    pub p2p_port: u16,
    pub local: bool,
    pub gateway: Option<String>,
    pub contract_wasm: Vec<u8>,
    pub params: Vec<u8>,
    pub own_id: boxes::PlayerId,
    pub own_entry: roster::PeerEntry,
}
