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
    "lele:build".exec = "cargo build --all-targets";
    "lele:clippy".exec = "cargo clippy --all-targets -- -D warnings";
    "lele:fmt".exec = "cargo fmt -- --check";
    "lele:nextest".exec = "cargo nextest run --all-targets";
    "lele:lint".exec = "cargo run --manifest-path ../lele_lint/Cargo.toml";
  };

  git-hooks.hooks = {};
}
