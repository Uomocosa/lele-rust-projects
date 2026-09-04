use freenet_stdlib::prelude::{ContractInstanceId, ContractKey};

#[must_use]
pub fn instance_id_from_key(key: &ContractKey) -> ContractInstanceId {
    *key.id()
}

#[cfg(test)]
mod tests {
    use super::instance_id_from_key;
    use crate::freenet_client_contract_key_from_wasm::contract_key_from_wasm;

    #[test]
    fn test_usage() {
        let k = contract_key_from_wasm(&[1, 2, 3], "a");
        let _ = instance_id_from_key(&k);
    }
}
