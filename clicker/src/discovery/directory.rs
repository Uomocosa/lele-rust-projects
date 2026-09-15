use super::directory_bridge_tick;
use super::directory_poll;
use super::directory_publish_room;
use crate::discovery;
use freenet_stdlib::prelude::*;

pub struct Directory {
    pub(crate) client: discovery::Client,
    pub contract_key: ContractKey,
    pub(crate) contract: ContractContainer,
    pub(crate) slots: discovery::DirectoryState,
}

#[rustfmt::skip]
impl Directory {
    /// # Errors
    /// Returns `Error` if the directory publish update fails.
    pub fn publish_room(&self, room: &str, params: &[u8], peer_id: &str, addrs: &[String]) -> Result<(), discovery::Error> { directory_publish_room::publish_room(self, room, params, peer_id, addrs) }
    /// # Errors
    /// Returns `Error` if polling the directory stream fails.
    pub async fn poll(&mut self) -> Result<discovery::DirectoryState, discovery::Error> { directory_poll::poll(self).await }
    /// # Errors
    /// Returns `Error` if the directory bridge fails.
    pub fn bridge_tick(&self, last_bridge: &mut Option<std::time::Instant>, now: std::time::Instant) -> Result<(), discovery::Error> { directory_bridge_tick::directory_bridge_tick(self, last_bridge, now) }
}
// no test_usage necessary
