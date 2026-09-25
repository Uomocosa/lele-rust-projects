use std::time::Instant;

use atomic_delegate_macros::atomic_delegate;
use freenet_stdlib::prelude::{ContractContainer, ContractKey};

use super::super::membership::peer_entry::PeerEntry;
use super::super::membership::roster_state::RosterState;
use super::super::params::player_id::PlayerId;
use super::super::params::remote_peer_id::RemotePeerId;
use super::client::Client;
use crate::discovery;

pub struct RosterClient {
    pub(crate) client: Client,
    pub contract_key: ContractKey,
    pub(crate) contract: ContractContainer,
    pub(crate) slots: RosterState,
    pub own: PlayerId,
    pub(crate) peer_id: RemotePeerId,
    pub addrs: Vec<String>,
    pub(crate) foreign_seen: Option<Instant>,
    pub(crate) foreign_sum: u64,
    pub(crate) last_bridge: Option<Instant>,
}

#[atomic_delegate]
impl RosterClient {
    pub async fn connect(
        host: &str,
        port: u16,
        contract_wasm: &[u8],
        params: &[u8],
        own: PlayerId,
        peer_id: &RemotePeerId,
        addrs: &[String],
    ) -> Result<Self, discovery::Error> {
    }
    pub fn announce(&self) -> Result<(), discovery::Error> {}
    pub async fn poll(&mut self) -> Result<Vec<PeerEntry>, discovery::Error> {}
    pub fn bridge_tick(&mut self, now: Instant) -> Result<(), discovery::Error> {}
}
// no test_usage necessary
