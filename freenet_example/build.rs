use std::process::Command;

fn main() {
    let dst = "contract/clicker_contract.wasm";
    println!("cargo:rerun-if-changed=contract/src/lib.rs");
    println!("cargo:rerun-if-changed=contract/Cargo.toml");
    println!("cargo:rerun-if-changed={dst}");

    if std::path::Path::new(dst).exists() {
        return;
    }

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
    std::fs::copy(wasm_src, dst).expect("failed to copy contract WASM");
}
