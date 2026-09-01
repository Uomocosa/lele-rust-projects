use crate::clicker_client_method;
use crate::clicker_error;
use crate::freenet_client;
use std::collections::BTreeMap;
use std::time::Instant;

use freenet_stdlib::prelude::*;

use crate::Role;
use freenet_client::FreenetClient;

pub struct ClickerClient {
    pub(crate) client: FreenetClient,
    pub contract_key: ContractKey,
    pub(crate) slots: BTreeMap<u64, u64>,
    pub tag: u64,
    pub(crate) foreign_seen: Option<Instant>,
    pub(crate) foreign_sum: u64,
    pub(crate) last_bridge: Option<Instant>,
    pub(crate) contract: ContractContainer,
}

#[rustfmt::skip]
impl ClickerClient {
    /// # Errors
    /// Returns `ClickerError` if the connection or contract deployment fails.
    pub async fn connect(host: &str, port: u16, contract_wasm: &[u8], role: Role) -> Result<Self, clicker_error::ClickerError> {
        clicker_client_method::connect(host, port, contract_wasm, &[], role, 0).await
    }
    /// # Errors
    /// Returns `ClickerError` if the connection or contract deployment fails.
    pub async fn connect_with_params(host: &str, port: u16, contract_wasm: &[u8], params: &[u8], role: Role) -> Result<Self, clicker_error::ClickerError> {
        clicker_client_method::connect(host, port, contract_wasm, params, role, 0).await
    }
    /// # Errors
    /// Returns `ClickerError` if the connection or contract deployment fails.
    pub async fn connect_with_tag(host: &str, port: u16, contract_wasm: &[u8], params: &[u8], role: Role, tag: u64) -> Result<Self, clicker_error::ClickerError> {
        clicker_client_method::connect(host, port, contract_wasm, params, role, tag).await
    }
    #[must_use]
    pub fn count(&self) -> u64 {
        clicker_client_method::count(self)
    }
    #[must_use]
    pub fn own(&self) -> u64 {
        clicker_client_method::own(self)
    }
    /// # Errors
    /// Returns `ClickerError` if the get request or deserialization fails.
    pub async fn state(&mut self) -> Result<u64, clicker_error::ClickerError> {
        clicker_client_method::state(self).await
    }
    /// # Errors
    /// Returns `ClickerError` if the update fails or the response is unexpected.
    pub async fn tick(&mut self) -> Result<u64, clicker_error::ClickerError> {
        clicker_client_method::tick(self).await
    }
    pub fn note_foreign_slots(&mut self) {
        clicker_client_method::note_foreign_slots(self);
    }
    #[must_use]
    pub fn foreign_tags(&self) -> Vec<u64> {
        clicker_client_method::foreign_tags(self)
    }
    /// # Errors
    /// Returns `ClickerError` if the bridge subscribe or re-put fails.
    pub async fn bridge_tick(&mut self) -> Result<(), clicker_error::ClickerError> {
        clicker_client_method::bridge_tick(self, std::time::Instant::now()).await
    }
    /// # Errors
    /// Returns `ClickerError` if the bridge subscribe or re-put fails.
    pub async fn bridge_tick_at(&mut self, now: std::time::Instant) -> Result<(), clicker_error::ClickerError> {
        clicker_client_method::bridge_tick(self, now).await
    }
}

// no test_usage necessary — exercised via integration tests
