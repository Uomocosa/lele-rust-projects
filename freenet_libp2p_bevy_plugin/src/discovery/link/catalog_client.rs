use std::time::Instant;

use atomic_delegate_macros::atomic_delegate;
use freenet_stdlib::prelude::{ContractContainer, ContractKey};

use super::super::directory::room_catalog::RoomCatalog;
use super::super::params::contract_params::ContractParams;
use super::super::params::remote_peer_id::RemotePeerId;
use super::super::params::room_name::RoomName;
use super::client::Client;
use crate::discovery;

pub struct CatalogClient {
    pub(crate) client: Client,
    pub contract_key: ContractKey,
    pub(crate) contract: ContractContainer,
    pub(crate) slots: RoomCatalog,
    pub(crate) last_bridge: Option<Instant>,
}

#[atomic_delegate]
impl CatalogClient {
    pub async fn connect(
        host: &str,
        port: u16,
        contract_wasm: &[u8],
        params: &[u8],
    ) -> Result<Self, discovery::Error> {
    }
    pub async fn poll(&mut self) -> Result<RoomCatalog, discovery::Error> {}
    pub fn publish_room(
        &self,
        room: &RoomName,
        params: &ContractParams,
        peer_id: &RemotePeerId,
        addrs: &[String],
    ) -> Result<(), discovery::Error> {
    }
    pub fn bridge_tick(&mut self, now: Instant) -> Result<(), discovery::Error> {}
}
// no test_usage necessary
