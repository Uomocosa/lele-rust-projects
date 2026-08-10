pub fn load_wasm() -> Vec<u8> {
    include_bytes!("../../../contract/roster_contract.wasm").to_vec()
}
