use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=contract/src/lib.rs");
    println!("cargo:rerun-if-changed=contract/Cargo.toml");

    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--manifest-path",
            "contract/Cargo.toml",
        ])
        .status()
        .expect("failed to build contract WASM");

    if !status.success() {
        panic!("contract WASM build failed");
    }

    let wasm_src = "contract/target/wasm32-unknown-unknown/release/clicker_contract.wasm";
    let wasm_dst = "contract/clicker_contract.wasm";
    std::fs::copy(wasm_src, wasm_dst).expect("failed to copy contract WASM");
}
