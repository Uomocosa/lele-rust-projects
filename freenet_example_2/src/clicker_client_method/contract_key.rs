use crate::clicker_client;
use freenet_stdlib::prelude::ContractKey;

pub fn contract_key(client: &clicker_client::ClickerClient) -> ContractKey {
    client.contract_key
}

// no test_usage necessary — exercised via integration tests
