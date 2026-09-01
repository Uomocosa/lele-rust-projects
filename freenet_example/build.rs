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
    let target_dir =
        std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "contract/target".to_string());

    let counter_proto = [
        "contract/src/lib.rs",
        "contract/Cargo.toml",
        "contract/Cargo.lock",
    ];
    let counter_out = "contract/clicker_contract.wasm";
    for p in counter_proto {
        println!("cargo:rerun-if-changed={p}");
    }
    println!("cargo:rerun-if-changed={counter_out}");
    if needs_build(&counter_proto, counter_out) {
        build_contract(
            "contract/Cargo.toml",
            "clicker_contract.wasm",
            &target_dir,
            counter_out,
        );
    }

    let set_proto = [
        "contract/set_contract/src/lib.rs",
        "contract/set_contract/Cargo.toml",
    ];
    let set_out = "contract/set_contract.wasm";
    for p in set_proto {
        println!("cargo:rerun-if-changed={p}");
    }
    println!("cargo:rerun-if-changed={set_out}");
    if needs_build(&set_proto, set_out) {
        build_contract(
            "contract/set_contract/Cargo.toml",
            "set_contract.wasm",
            &target_dir,
            set_out,
        );
    }
}
