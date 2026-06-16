use freenet_stdlib::prelude::*;

use crate::FreenetClient;
use crate::Role;

pub struct ClickerClient {
    pub(crate) client: FreenetClient,
    pub(crate) contract_key: ContractKey,
    pub(crate) count: u64,
}

#[rustfmt::skip]
impl ClickerClient {
    pub async fn connect(host: &str, port: u16, contract_wasm: &[u8], role: Role) -> Result<Self, crate::ClickerError> {
        crate::ClickerClientMethod::connect(host, port, contract_wasm, role).await
    }
    pub fn contract_key(&self) -> ContractKey {
        crate::ClickerClientMethod::contract_key(self)
    }
    pub fn count(&self) -> u64 {
        crate::ClickerClientMethod::count(self)
    }
    pub async fn state(&mut self) -> Result<u64, crate::ClickerError> {
        crate::ClickerClientMethod::state(self).await
    }
    pub async fn tick(&mut self) -> Result<u64, crate::ClickerError> {
        crate::ClickerClientMethod::tick(self).await
    }
}
