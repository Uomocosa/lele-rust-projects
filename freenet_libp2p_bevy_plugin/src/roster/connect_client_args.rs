use std::time::Duration;

use crate::net_id;
use crate::roster;

pub struct ConnectClientArgs<'a> {
    pub host: &'a str,
    pub port: u16,
    pub contract_wasm: &'a [u8],
    pub params: &'a [u8],
    pub own_id: net_id::NetworkId,
    pub own_entry: roster::PeerEntry,
    pub not_found_grace: Duration,
}
