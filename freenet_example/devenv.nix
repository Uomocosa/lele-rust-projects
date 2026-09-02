{ pkgs, lib, config, inputs, ... }: {
  stdenv = pkgs.gccStdenv;

  languages.rust = {
    enable = true;
    channel = "stable";
    components = [ "rustc" "cargo" "clippy" "rustfmt" ];
    targets = [ "wasm32-unknown-unknown" ];
  };

  packages = with pkgs; [
    cargo-nextest
    clang
    pkg-config
    gnumake
    glibc.dev
    linuxHeaders
    (if pkgs ? ffmpeg-full then pkgs.ffmpeg-full else ffmpeg)
    xorg.xdpyinfo
    xterm
    wmctrl
    xdotool
  ];

  env.CARGO_TARGET_DIR = "/tmp/frt-build";
  env.C_INCLUDE_PATH = "${pkgs.glibc.dev}/include:${pkgs.linuxHeaders}/include";
  env.CFLAGS = "-I${pkgs.glibc.dev}/include -Wno-error";
  env.CPPFLAGS = "-I${pkgs.glibc.dev}/include -Wno-error";

  tasks = {
    "lele:build".exec = "cargo build --all-targets --features dev";
    "lele:clippy".exec = "cargo clippy --all-targets --features dev -- -D warnings";
    "lele:fmt".exec = "cargo fmt -- --check";
    "lele:nextest".exec = "cargo nextest run --all-targets --features dev";
    "lele:lint".exec = "cargo run --manifest-path ../lele_lint/Cargo.toml";
    "lele:taxonomy_check".exec = "cargo run --manifest-path ../lele_function_taxonomy/Cargo.toml --features rustc-private -- --manifest-path ./Cargo.toml";
    "freenet:contract-harness".exec = "cargo test --manifest-path ../freenet_contract_harness/Cargo.toml -- --nocapture";
    "freenet:run-local-mainnet".exec = "cargo nextest run --test mainnet_local --features dev --run-ignored all -- --nocapture";
    "freenet:run-cross-os".exec = "cargo nextest run --test mainnet_cross --features dev --run-ignored all -- --nocapture";
  };

  git-hooks.hooks = {};
}
