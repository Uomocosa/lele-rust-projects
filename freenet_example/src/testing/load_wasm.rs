use std::io;

const WASM_PATH: &str = "contract/target/wasm32-unknown-unknown/release/clicker_contract.wasm";

pub fn load_wasm() -> Result<Vec<u8>, io::Error> {
    std::fs::read(WASM_PATH)
}
