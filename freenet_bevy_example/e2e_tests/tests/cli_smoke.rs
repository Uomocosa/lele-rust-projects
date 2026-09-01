use std::path::PathBuf;
use std::time::Duration;

use e2e_tests::{expect_line, spawn_binary};

fn binary_path() -> PathBuf {
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            manifest.parent().unwrap().join("target")
        });
    let release = target_dir.join("release").join("freenet-bevy");
    if release.exists() {
        return release;
    }
    target_dir.join("debug").join("freenet-bevy")
}

#[test]
fn test_cli_binary_starts() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "binary not found at {}. build with: cargo build",
        bin.display()
    );

    let (mut child, receivers) =
        spawn_binary(bin.to_str().unwrap(), &["--mode", "cli"]);
    let stdout_rx = &receivers[0];

    let connected =
        expect_line(stdout_rx, Duration::from_secs(600), "connected, running");
    assert!(!connected.is_empty(), "should print connected line");

    let _ = child.kill();
    let _ = child.wait();
}
