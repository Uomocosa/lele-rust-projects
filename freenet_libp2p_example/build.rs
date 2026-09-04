#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::manual_assert)]

use std::path::Path;
use std::process::Command;

fn needs_build(proto: &[&str], out: &str) -> bool {
    let out_path = Path::new(out);
    if !out_path.exists() {
        return true;
    }
    let Ok(out_time) = out_path.metadata().and_then(|m| m.modified()) else {
        return true;
    };
    proto.iter().any(|src| {
        Path::new(src)
            .metadata()
            .and_then(|m| m.modified())
            .map_or(true, |t| t > out_time)
    })
}

fn build_contract(manifest: &str, crate_name: &str, target_dir: &str, out: &str) {
    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--target-dir",
            target_dir,
            "--manifest-path",
            manifest,
        ])
        .status()
        .expect("failed to build contract WASM");
    if !status.success() {
        panic!("contract WASM build failed for {manifest}");
    }
    let wasm_src = format!("{target_dir}/wasm32-unknown-unknown/release/{crate_name}");
    std::fs::copy(&wasm_src, out).expect("failed to copy contract WASM");
}

fn main() {
    let wasm_target_dir = "contract/target".to_string();
    let proto = [
        "contract/src/lib.rs",
        "contract/Cargo.toml",
        "contract/Cargo.lock",
    ];
    let out = "contract/letter_contract.wasm";
    for p in proto {
        println!("cargo:rerun-if-changed={p}");
    }
    println!("cargo:rerun-if-changed={out}");
    if needs_build(&proto, out) {
        build_contract(
            "contract/Cargo.toml",
            "letter_contract.wasm",
            &wasm_target_dir,
            out,
        );
    }
}
