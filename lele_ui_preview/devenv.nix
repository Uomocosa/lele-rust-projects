{ pkgs, lib, config, inputs, ... }: {
  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [ "rustc" "cargo" "clippy" "rustfmt" ];
  };

  packages = with pkgs; [
    cargo-nextest
  ];

  env.CARGO_TARGET_DIR = "/tmp/frt-build";

  tasks = {
    "lele:build" = { exec = "cargo build --all-targets --all-features"; showOutput = true; };
    "lele:clippy" = { exec = "cargo clippy --all-targets --all-features -- -D warnings"; showOutput = true; };
    "lele:fmt" = { exec = "cargo fmt -- --check"; showOutput = true; };
    "lele:nextest" = { exec = "cargo nextest run --all-targets --all-features"; showOutput = true; };
    "lele:lint" = { exec = "cargo run --manifest-path ../lele_lint/Cargo.toml"; showOutput = true; };
    # Generate ../lele_code_viewer/ui_preview (web driver, headless Chrome).
    "ui:preview:viewer" = { exec = "cargo run --release -- ../lele_code_viewer"; showOutput = true; };
    # Generate ../freenet_libp2p_bevy_plugin/ui_preview (Bevy driver over BRP; opens a window).
    "ui:preview:plugin" = { exec = "cargo run --release -- ../freenet_libp2p_bevy_plugin"; showOutput = true; };
  };
}
