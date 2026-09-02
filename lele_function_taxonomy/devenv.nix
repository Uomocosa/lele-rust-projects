{ pkgs, lib, config, inputs, ... }: {
  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
  };

  packages = with pkgs; [
    cargo-nextest
  ];

  env.CARGO_TARGET_DIR = "/tmp/frt-build";

  tasks = {
    "lele:build".exec = "cargo build --features rustc-private --all-targets";
    "lele:clippy".exec = "cargo clippy --features rustc-private --all-targets -- -D warnings";
    "lele:fmt".exec = "cargo fmt -- --check";
    "lele:nextest".exec = "cargo nextest run --features rustc-private --all-targets";
    "lele:taxonomy".exec = "cargo run -p lele_function_taxonomy --features rustc-private -- --manifest-path ./Cargo.toml";
    "lele:lint".exec = "cargo run --features rustc-private -- --manifest-path ./Cargo.toml";
  };

  git-hooks.hooks = {};
}
