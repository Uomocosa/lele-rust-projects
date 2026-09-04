use std::sync::Arc;

use freenet_stdlib::prelude::{
    ContractCode, ContractContainer, ContractKey, ContractWasmAPIVersion, Parameters,
    WrappedContract,
};

#[must_use]
pub fn contract_key_from_wasm(wasm: &[u8], lobby: &str) -> ContractKey {
    let serialized = bincode::serialize(&lobby.to_string()).unwrap_or_default();
    let params = Parameters::from(serialized);
    let code = Arc::new(ContractCode::from(wasm.to_vec()));
    let wrapped = WrappedContract::new(code, params);
    let _container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped.clone()));
    wrapped.key
}

#[cfg(test)]
mod tests {
    use super::contract_key_from_wasm;

    #[test]
    fn test_usage() {
        let wasm = vec![0u8; 4];
        let _ = contract_key_from_wasm(&wasm, "lobby");
    }
}
