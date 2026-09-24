use std::time::Instant;

use atomic_delegate_macros::atomic_delegate;
use freenet_stdlib::prelude::{ContractContainer, ContractKey};

use super::client::Client;
use super::peer_entry::PeerEntry;
use super::player_id::PlayerId;
use super::roster_state::RosterState;
use crate::discovery;

pub struct Roster {
    pub(crate) client: Client,
    pub contract_key: ContractKey,
    pub(crate) contract: ContractContainer,
    pub(crate) slots: RosterState,
    pub own: PlayerId,
    pub(crate) peer_id: String,
    pub addrs: Vec<String>,
    pub(crate) foreign_seen: Option<Instant>,
    pub(crate) foreign_sum: u64,
    pub(crate) last_bridge: Option<Instant>,
}

#[atomic_delegate]
impl Roster {
    pub async fn connect(
        host: &str,
        port: u16,
        contract_wasm: &[u8],
        params: &[u8],
        own: PlayerId,
        peer_id: &str,
        addrs: &[String],
    ) -> Result<Self, discovery::Error> {
    }
    pub fn announce(&self) -> Result<(), discovery::Error> {}
    pub async fn poll(&mut self) -> Result<Vec<PeerEntry>, discovery::Error> {}
    pub fn bridge_tick(&mut self, now: Instant) -> Result<(), discovery::Error> {}
}
// no test_usage necessary
