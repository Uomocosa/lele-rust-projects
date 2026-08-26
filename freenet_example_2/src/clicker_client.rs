use crate::clicker_client_method;
use crate::clicker_error;
use crate::freenet_client;
use freenet_stdlib::prelude::*;

use crate::Role;
use freenet_client::FreenetClient;

pub struct ClickerClient {
    pub(crate) client: FreenetClient,
    pub(crate) contract_key: ContractKey,
    pub(crate) count: u64,
}

#[rustfmt::skip]
impl ClickerClient {
    pub async fn connect(host: &str, port: u16, contract_wasm: &[u8], role: Role) -> Result<Self, clicker_error::ClickerError> {
        clicker_client_method::connect(host, port, contract_wasm, &[], role).await
    }
    pub async fn connect_with_params(host: &str, port: u16, contract_wasm: &[u8], params: &[u8], role: Role) -> Result<Self, clicker_error::ClickerError> {
        clicker_client_method::connect(host, port, contract_wasm, params, role).await
    }
    pub fn contract_key(&self) -> ContractKey {
        clicker_client_method::contract_key(self)
    }
    pub fn count(&self) -> u64 {
        clicker_client_method::count(self)
    }
    pub async fn state(&mut self) -> Result<u64, clicker_error::ClickerError> {
        clicker_client_method::state(self).await
    }
    pub async fn tick(&mut self) -> Result<u64, clicker_error::ClickerError> {
        clicker_client_method::tick(self).await
    }
}

// no test_usage necessary — exercised via integration tests
