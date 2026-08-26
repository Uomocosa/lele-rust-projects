use crate::freenet_client;
use std::collections::BTreeSet;

use freenet_stdlib::prelude::*;

use super::set_client_method;
use crate::clicker_error;
use freenet_client::FreenetClient;

pub struct SetClient {
    pub(crate) client: FreenetClient,
    pub(crate) contract_key: ContractKey,
    pub(crate) set: BTreeSet<u64>,
    pub tag: u64,
    pub(crate) seq: u64,
}

#[rustfmt::skip]
impl SetClient {
    pub async fn connect(host: &str, port: u16, contract_wasm: &[u8], params: &[u8], tag: u64) -> Result<Self, clicker_error::ClickerError> {
        set_client_method::connect(host, port, contract_wasm, params, tag).await
    }
    pub fn contract_key(&self) -> ContractKey {
        set_client_method::contract_key(self)
    }
    pub fn count(&self) -> u64 {
        set_client_method::state_len(self)
    }
    pub fn own_count(&self) -> u64 {
        set_client_method::own_count(self)
    }
    pub async fn tick(&mut self) -> Result<u64, clicker_error::ClickerError> {
        set_client_method::tick(self).await
    }
}

// no test_usage necessary — thin-delegate struct file, coverage via e2e
