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
    "lele:build" = { exec = "cargo build --all-targets --features dev"; showOutput = true; };
    "lele:clippy" = { exec = "cargo clippy --all-targets --features dev -- -D warnings"; showOutput = true; };
    "lele:fmt" = { exec = "cargo fmt -- --check"; showOutput = true; };
    "lele:nextest" = { exec = "cargo nextest run --all-targets --features dev"; showOutput = true; };
    "lele:lint" = { exec = "cargo run --manifest-path ../lele_lint/Cargo.toml"; showOutput = true; };
    "lele:taxonomy_check" = { exec = "cargo run --manifest-path ../lele_function_taxonomy/Cargo.toml --features rustc-private -- --manifest-path ./Cargo.toml"; showOutput = true; };
    "freenet:contract-harness" = { exec = "cargo test --manifest-path ../freenet_contract_harness/Cargo.toml -- --nocapture"; showOutput = true; };
    "freenet:run-local-mainnet" = { exec = "cargo nextest run --test mainnet_local --features dev --run-ignored all -- --nocapture"; showOutput = true; };
    "freenet:run-cross-os" = { exec = "cargo nextest run --test mainnet_cross --features dev --run-ignored all -- --nocapture"; showOutput = true; };
  };

  git-hooks.hooks = {
    lele-clippy = {
      enable = true;
      name = "clippy (freenet_example)";
      entry = "bash -c 'cd freenet_example && devenv tasks run lele:clippy 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-fmt = {
      enable = true;
      name = "fmt (freenet_example)";
      entry = "bash -c 'cd freenet_example && devenv tasks run lele:fmt 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-lint = {
      enable = true;
      name = "lele_lint (freenet_example)";
      entry = "bash -c 'cd freenet_example && devenv tasks run lele:lint 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-taxonomy = {
      enable = true;
      name = "taxonomy_check (freenet_example)";
      entry = "bash -c 'cd freenet_example && devenv tasks run lele:taxonomy_check 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    freenet-contract-harness = {
      enable = true;
      name = "contract-harness (freenet_example)";
      entry = "bash -c 'cd freenet_example && devenv tasks run freenet:contract-harness 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
  };
}
