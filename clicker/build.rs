#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::manual_assert)]

use std::process::Command;

fn main() {
    link_x11();
    let dst = "contract/roster_contract.wasm";
    println!("cargo:rerun-if-changed=contract/src/lib.rs");
    println!("cargo:rerun-if-changed=contract/Cargo.toml");
    println!("cargo:rerun-if-changed=contract/Cargo.lock");
    println!("cargo:rerun-if-changed={dst}");

    let src_files = [
        "contract/src/lib.rs",
        "contract/Cargo.toml",
        "contract/Cargo.lock",
    ];
    if std::path::Path::new(dst).exists()
        && let Ok(wasm_meta) = std::fs::metadata(dst)
        && let Ok(wasm_time) = wasm_meta.modified()
    {
        let all_fresh = src_files.iter().all(|src| {
            std::fs::metadata(src)
                .and_then(|m| m.modified())
                .is_ok_and(|t| t <= wasm_time)
        });
        if all_fresh {
            return;
        }
    }

    let wasm_target_dir = "contract/target".to_string();

    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--target-dir",
            &wasm_target_dir,
            "--manifest-path",
            "contract/Cargo.toml",
        ])
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .status()
        .expect("failed to build contract WASM");

    if !status.success() {
        panic!("contract WASM build failed");
    }

    let wasm_src = format!("{wasm_target_dir}/wasm32-unknown-unknown/release/roster_contract.wasm");
    std::fs::copy(&wasm_src, dst).expect("failed to copy contract WASM");
}

fn link_x11() {
    println!("cargo:rerun-if-env-changed=CLICKER_LINK_X11");
    if std::env::var("CLICKER_LINK_X11").is_ok_and(|v| v == "0") {
        return;
    }
    println!("cargo:rustc-link-arg=-Wl,--no-as-needed");
    for lib in [
        "x11",
        "xext",
        "xrandr",
        "xcursor",
        "xi",
        "xfixes",
        "xrender",
        "xkbcommon-x11",
        "vulkan",
    ] {
        link_pkg_config(lib);
    }
    println!("cargo:rustc-link-arg=-Wl,--as-needed");
}

fn link_pkg_config(lib: &str) {
    let Ok(output) = Command::new("pkg-config")
        .args(["--libs-only-L", "--libs-only-l", lib])
        .output()
    else {
        return;
    };
    if !output.status.success() {
        return;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for token in stdout.split_whitespace() {
        if let Some(dir) = token.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={dir}");
        } else if let Some(name) = token.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={name}");
        }
    }
}
