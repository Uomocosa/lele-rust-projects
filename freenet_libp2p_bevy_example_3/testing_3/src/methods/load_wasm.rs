pub fn load_wasm() -> Vec<u8> {
    include_bytes!("../../../contract/membership_contract.wasm").to_vec()
}
