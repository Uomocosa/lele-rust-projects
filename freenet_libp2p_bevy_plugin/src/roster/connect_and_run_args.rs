use crate::net_id;
use crate::roster;

pub struct ConnectAndRunArgs {
    pub local: bool,
    pub gateway: Option<String>,
    pub contract_wasm: Vec<u8>,
    pub params: Vec<u8>,
    pub own_id: net_id::NetworkId,
    pub own_entry: roster::PeerEntry,
}
