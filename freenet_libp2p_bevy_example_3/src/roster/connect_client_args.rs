use std::time::Duration;

use libp2p::identity::Keypair;

use crate::boxes;
use crate::roster;

pub struct ConnectClientArgs<'a> {
    pub host: &'a str,
    pub port: u16,
    pub contract_wasm: &'a [u8],
    pub params: &'a [u8],
    pub own_keypair: Keypair,
    pub own_id: boxes::PlayerId,
    pub own_entry: roster::PeerEntry,
    pub not_found_grace: Duration,
}
