use freenet_stdlib::prelude::ContractKey;

pub fn contract_key(client: &crate::ClickerClient) -> ContractKey {
    client.contract_key
}
