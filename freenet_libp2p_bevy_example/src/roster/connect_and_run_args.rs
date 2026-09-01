use libp2p::identity::Keypair;

use crate::boxes;
use crate::roster;

pub struct ConnectAndRunArgs {
    pub local: bool,
    pub gateway: Option<String>,
    pub contract_wasm: Vec<u8>,
    pub params: Vec<u8>,
    pub own_keypair: Keypair,
    pub own_id: boxes::PlayerId,
    pub own_entry: roster::PeerEntry,
}
