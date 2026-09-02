{ pkgs, lib, config, inputs, ... }: {
  languages.rust = {
    enable = true;
    channel = "stable";
    components = [ "rustc" "cargo" "clippy" "rustfmt" ];
  };

  packages = with pkgs; [
    cargo-nextest
  ];

  env.CARGO_TARGET_DIR = "/tmp/frt-build";

  tasks = {
    "lele:build" = { exec = "cargo build --all-targets"; showOutput = true; };
    "lele:clippy" = { exec = "cargo clippy --all-targets -- -D warnings"; showOutput = true; };
    "lele:fmt" = { exec = "cargo fmt -- --check"; showOutput = true; };
    "lele:nextest" = { exec = "cargo nextest run --all-targets"; showOutput = true; };
    "lele:lint" = { exec = "cargo run --manifest-path ../lele_lint/Cargo.toml"; showOutput = true; };
  };

  git-hooks.hooks = {};
}
