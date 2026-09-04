use std::collections::BTreeMap;
use std::time::Instant;

use crate::discovery_bridge_tick;
use crate::discovery_connect;
use crate::discovery_last_next;
use crate::discovery_next_seq;
use crate::discovery_poll;
use crate::discovery_publish_frame;
use crate::discovery_publish_peer;
use crate::frame;
use crate::freenet_client;

pub type Pubkey = [u8; 32];
pub use crate::discovery_chain_entry::ChainEntry;
pub use crate::discovery_peer_record::PeerRecord;
pub use crate::discovery_state_data::StateData;

pub struct Discovery {
    pub client: freenet_client::FreenetClient,
    pub key: freenet_stdlib::prelude::ContractKey,
    pub lobby: String,
    pub peers: BTreeMap<Pubkey, PeerRecord>,
    pub chain: BTreeMap<u64, ChainEntry>,
    pub last_bridge: Option<Instant>,
    pub foreign_len: usize,
    pub contract_wasm: Vec<u8>,
}

#[rustfmt::skip]
impl Discovery {
    /// # Errors
    /// Returns error if connection fails.
    ///
    /// # Panics
    /// May panic if serialization fails.
    pub async fn connect(host: &str, port: u16, wasm: &[u8], lobby: &str) -> Result<Self, String> { discovery_connect::connect(host, port, wasm, lobby).await }
    #[must_use]
    pub fn next_seq(&self) -> u64 { discovery_next_seq::next_seq(self) }
    #[must_use]
    pub fn last_next(&self) -> u8 { discovery_last_next::last_next(self) }
    /// # Errors
    /// Returns error if send fails.
    ///
    /// # Panics
    /// May panic if serialization fails.
    pub async fn publish_frame(&mut self, frame: &frame::Frame) -> Result<(), String> { discovery_publish_frame::publish_frame(self, frame).await }
    /// # Errors
    /// Returns error if send fails.
    pub async fn publish_peer(&mut self, pubkey: Pubkey, signing: &ed25519_dalek::SigningKey, peer_id: Vec<u8>, addrs: Vec<String>) -> Result<(), String> { discovery_publish_peer::publish_peer(self, pubkey, signing, peer_id, addrs).await }
    pub async fn poll(&mut self) { discovery_poll::poll(self).await; }
    pub async fn bridge_tick(&mut self, now: Instant) { discovery_bridge_tick::bridge_tick(self, now).await; }
}

// no test_usage necessary
