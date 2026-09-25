use std::time::Instant;

use atomic_delegate_macros::atomic_delegate;
use freenet_stdlib::prelude::{ContractContainer, ContractKey};

use super::super::directory::directory_state::DirectoryState;
use super::client::Client;
use crate::discovery;

pub struct DirectoryClient {
    pub(crate) client: Client,
    pub contract_key: ContractKey,
    pub(crate) contract: ContractContainer,
    pub(crate) slots: DirectoryState,
    pub(crate) last_bridge: Option<Instant>,
}

#[atomic_delegate]
impl DirectoryClient {
    pub async fn connect(
        host: &str,
        port: u16,
        contract_wasm: &[u8],
        params: &[u8],
    ) -> Result<Self, discovery::Error> {
    }
    pub async fn poll(&mut self) -> Result<DirectoryState, discovery::Error> {}
    pub fn publish_room(
        &self,
        room: &str,
        params: &[u8],
        peer_id: &str,
        addrs: &[String],
    ) -> Result<(), discovery::Error> {
    }
    pub fn bridge_tick(&mut self, now: Instant) -> Result<(), discovery::Error> {}
}
// no test_usage necessary
