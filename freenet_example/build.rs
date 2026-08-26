use std::process::Command;

fn main() {
    let dst = "contract/clicker_contract.wasm";
    println!("cargo:rerun-if-changed=contract/src/lib.rs");
    println!("cargo:rerun-if-changed=contract/Cargo.toml");
    println!("cargo:rerun-if-changed=contract/Cargo.lock");
    println!("cargo:rerun-if-changed={dst}");

    let src_files = [
        "contract/src/lib.rs",
        "contract/Cargo.toml",
        "contract/Cargo.lock",
    ];
    if std::path::Path::new(dst).exists() {
        if let Ok(wasm_meta) = std::fs::metadata(dst) {
            if let Ok(wasm_time) = wasm_meta.modified() {
                let all_fresh = src_files.iter().all(|src| {
                    std::fs::metadata(src)
                        .and_then(|m| m.modified())
                        .map(|t| t <= wasm_time)
                        .unwrap_or(false)
                });
                if all_fresh {
                    return;
                }
            }
        }
    }

    let target_dir =
        std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "contract/target".to_string());

    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--target-dir",
            &target_dir,
            "--manifest-path",
            "contract/Cargo.toml",
        ])
        .status()
        .expect("failed to build contract WASM");

    if !status.success() {
        panic!("contract WASM build failed");
    }

    let wasm_src = format!("{target_dir}/wasm32-unknown-unknown/release/clicker_contract.wasm");
    std::fs::copy(&wasm_src, dst).expect("failed to copy contract WASM");
}
