const WASM_PATH: &str = "contract/target/wasm32-unknown-unknown/release/clicker_contract.wasm";

pub fn load_wasm() -> Vec<u8> {
    std::fs::read(WASM_PATH)
        .expect("contract WASM not found — build with: cargo make build-contract")
}
