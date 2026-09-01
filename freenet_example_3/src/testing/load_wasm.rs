#[must_use]
pub fn load_wasm() -> Vec<u8> {
    include_bytes!("../../contract/clicker_contract.wasm").to_vec()
}

// no test_usage necessary — exercised via integration tests
