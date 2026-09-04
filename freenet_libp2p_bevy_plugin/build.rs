#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    clippy::collapsible_if,
    clippy::manual_assert
)]
use std::process::Command;

fn needs_build(src: &[&str], dst: &str) -> bool {
    let Ok(wasm_meta) = std::fs::metadata(dst).and_then(|m| m.modified()) else {
        return true;
    };
    for s in src {
        if let Ok(meta) = std::fs::metadata(s).and_then(|m| m.modified()) {
            if meta > wasm_meta {
                return true;
            }
        }
    }
    false
}

fn build_one(manifest: &str, out: &str, target_dir: &str) {
    let src = vec![
        manifest.to_string(),
        format!(
            "{}/Cargo.lock",
            manifest
                .trim_end_matches("Cargo.toml")
                .trim_end_matches('/')
        ),
    ];
    // handled via rerun-if-changed outside
    let _ = src;
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
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .status()
        .expect("failed to build contract WASM");
    if !status.success() {
        panic!("contract WASM build failed for {manifest}");
    }
    let wasm_src = format!(
        "{target_dir}/wasm32-unknown-unknown/release/{}.wasm",
        if manifest.contains("directory") {
            "directory_contract"
        } else {
            "board_contract"
        }
    );
    std::fs::copy(&wasm_src, out).expect("failed to copy contract WASM");
}

fn main() {
    println!("cargo:rerun-if-changed=contract/directory/src/lib.rs");
    println!("cargo:rerun-if-changed=contract/directory/Cargo.toml");
    println!("cargo:rerun-if-changed=contract/directory/Cargo.lock");
    println!("cargo:rerun-if-changed=contract/board/src/lib.rs");
    println!("cargo:rerun-if-changed=contract/board/Cargo.toml");
    println!("cargo:rerun-if-changed=contract/board/Cargo.lock");
    println!("cargo:rerun-if-changed=contract/directory/directory_contract.wasm");
    println!("cargo:rerun-if-changed=contract/board/board_contract.wasm");

    let dir_wasm = "contract/directory/directory_contract.wasm";
    let board_wasm = "contract/board/board_contract.wasm";

    let dir_src = [
        "contract/directory/src/lib.rs",
        "contract/directory/Cargo.toml",
    ];
    let board_src = ["contract/board/src/lib.rs", "contract/board/Cargo.toml"];

    let dir_needs = needs_build(&dir_src, dir_wasm);
    let board_needs = needs_build(&board_src, board_wasm);

    if !dir_needs && !board_needs {
        return;
    }

    if dir_needs {
        build_one(
            "contract/directory/Cargo.toml",
            dir_wasm,
            "contract/directory/target",
        );
    }
    if board_needs {
        build_one(
            "contract/board/Cargo.toml",
            board_wasm,
            "contract/board/target",
        );
    }
}
