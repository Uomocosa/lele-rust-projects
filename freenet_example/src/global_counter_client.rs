use crate::global_counter_client_method;
use std::collections::BTreeMap;
use std::time::Instant;

use freenet_stdlib::prelude::*;

use crate::Role;
use crate::freenet_client::FreenetClient;
use crate::global_counter_error::GlobalCounterError;

pub type Pubkey = [u8; 32];

pub struct GlobalCounterClient {
    pub(crate) client: FreenetClient,
    pub contract_key: ContractKey,
    pub(crate) slots: BTreeMap<Pubkey, u64>,
    pub tag: u64,
    pub pubkey: Pubkey,
    pub(crate) foreign_seen: Option<Instant>,
    pub(crate) foreign_sum: u64,
    pub(crate) last_bridge: Option<Instant>,
    pub(crate) contract: ContractContainer,
}

impl GlobalCounterClient {
    /// # Errors
    /// Returns `GlobalCounterError` if the connection or contract deployment fails.
    pub async fn connect(
        host: &str,
        port: u16,
        contract_wasm: &[u8],
        role: Role,
    ) -> Result<Self, GlobalCounterError> {
        global_counter_client_method::connect(host, port, contract_wasm, &[], role, 0).await
    }
    /// # Errors
    /// Returns `GlobalCounterError` if the connection or contract deployment fails.
    pub async fn connect_with_params(
        host: &str,
        port: u16,
        contract_wasm: &[u8],
        params: &[u8],
        role: Role,
    ) -> Result<Self, GlobalCounterError> {
        global_counter_client_method::connect(host, port, contract_wasm, params, role, 0).await
    }
    /// # Errors
    /// Returns `GlobalCounterError` if the connection or contract deployment fails.
    pub async fn connect_with_tag(
        host: &str,
        port: u16,
        contract_wasm: &[u8],
        params: &[u8],
        role: Role,
        tag: u64,
    ) -> Result<Self, GlobalCounterError> {
        global_counter_client_method::connect(host, port, contract_wasm, params, role, tag).await
    }
    /// # Errors
    /// Returns `GlobalCounterError` if the get request or deserialization fails.
    pub async fn state(&mut self) -> Result<u64, GlobalCounterError> {
        global_counter_client_method::state(self).await
    }
    /// # Errors
    /// Returns `GlobalCounterError` if the update fails or the response is unexpected.
    pub async fn tick(&mut self) -> Result<u64, GlobalCounterError> {
        global_counter_client_method::tick(self).await
    }
    pub fn note_foreign_slots(&mut self) {
        global_counter_client_method::note_foreign_slots(self);
    }
    /// # Errors
    /// Returns `GlobalCounterError` if the bridge subscribe or re-put fails.
    pub async fn bridge_tick(&mut self) -> Result<(), GlobalCounterError> {
        global_counter_client_method::bridge_tick(self, std::time::Instant::now()).await
    }
    /// # Errors
    /// Returns `GlobalCounterError` if the bridge subscribe or re-put fails.
    pub async fn bridge_tick_at(
        &mut self,
        now: std::time::Instant,
    ) -> Result<(), GlobalCounterError> {
        global_counter_client_method::bridge_tick(self, now).await
    }
}

#[rustfmt::skip]
impl GlobalCounterClient {
    #[must_use]
    pub fn count(&self) -> u64 {
        global_counter_client_method::count(self)
    }
    #[must_use]
    pub fn own(&self) -> u64 {
        global_counter_client_method::own(self)
    }
    #[must_use]
    pub fn foreign_tags(&self) -> Vec<Pubkey> {
        global_counter_client_method::foreign_tags(self)
    }
}

// no test_usage necessary — exercised via integration tests
