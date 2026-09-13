use super::bridge_tick;
use super::roster_announce;
use super::roster_poll;
use crate::discovery;
use freenet_stdlib::prelude::*;

pub struct Roster {
    pub(crate) client: discovery::Client,
    pub contract_key: ContractKey,
    pub(crate) slots: discovery::RosterState,
    pub own: discovery::PlayerId,
    pub(crate) peer_id: String,
    pub(crate) addrs: Vec<String>,
    pub(crate) foreign_seen: Option<std::time::Instant>,
    pub(crate) foreign_sum: u64,
    pub(crate) last_bridge: Option<std::time::Instant>,
}

#[rustfmt::skip]
impl Roster {
    /// # Errors
    /// Returns `Error` if the roster announce update fails.
    pub fn announce(&self) -> Result<(), discovery::Error> { roster_announce::announce(self) }
    /// # Errors
    /// Returns `Error` if polling the notification stream fails.
    pub async fn poll(&mut self) -> Result<Vec<discovery::PeerEntry>, discovery::Error> { roster_poll::poll(self).await }
    /// # Errors
    /// Returns `Error` if the bridge subscribe fails.
    pub async fn bridge_tick(&mut self, now: std::time::Instant) -> Result<(), discovery::Error> { bridge_tick::bridge_tick(self, now).await }
}
// no test_usage necessary
