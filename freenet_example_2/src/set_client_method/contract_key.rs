use freenet_stdlib::prelude::ContractKey;

use crate::set_client;

pub fn contract_key(client: &set_client::SetClient) -> ContractKey {
    client.contract_key
}

// no test_usage necessary
