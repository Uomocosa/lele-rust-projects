use std::collections::BTreeSet;

use freenet_stdlib::prelude::*;

use super::set_client_method;
use crate::freenet_client::FreenetClient;
use crate::global_counter_error::GlobalCounterError;

pub struct SetClient {
    pub(crate) client: FreenetClient,
    pub contract_key: ContractKey,
    pub(crate) set: BTreeSet<u64>,
    pub tag: u64,
    pub(crate) seq: u64,
}

impl SetClient {
    /// # Errors
    /// Returns `GlobalCounterError` if the connection or contract deployment fails.
    pub async fn connect(
        host: &str,
        port: u16,
        contract_wasm: &[u8],
        params: &[u8],
        tag: u64,
    ) -> Result<Self, GlobalCounterError> {
        set_client_method::connect(host, port, contract_wasm, params, tag).await
    }
    /// # Errors
    /// Returns `GlobalCounterError` if the update fails or the response is unexpected.
    pub async fn tick(&mut self) -> Result<u64, GlobalCounterError> {
        set_client_method::tick(self).await
    }
}

#[rustfmt::skip]
impl SetClient {
    #[must_use]
    pub fn count(&self) -> u64 {
        set_client_method::state_len(self)
    }
    #[must_use]
    pub const fn own_count(&self) -> u64 {
        set_client_method::own_count(self)
    }
}

// no test_usage necessary — thin-delegate struct file, coverage via e2e
