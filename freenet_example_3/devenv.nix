{ pkgs, lib, config, inputs, ... }: {
  stdenv = pkgs.gccStdenv;

  languages.rust = {
    enable = true;
    channel = "stable";
    components = [ "rustc" "cargo" "clippy" "rustfmt" ];
  };

  packages = with pkgs; [
    bacon
    cargo-nextest
    clang
    pkg-config
    gnumake
    glibc.dev
    linuxHeaders
    ffmpeg
    xorg.xdpyinfo
    xdotool
  ];

  env.CARGO_TARGET_DIR = "/tmp/frt-build";
  env.C_INCLUDE_PATH = "${pkgs.glibc.dev}/include:${pkgs.linuxHeaders}/include";
  env.CFLAGS = "-I${pkgs.glibc.dev}/include -Wno-error";
  env.CPPFLAGS = "-I${pkgs.glibc.dev}/include -Wno-error";

  tasks = {
    "lele:build".exec = ''
      set -e
      echo ">>> [lele:build] cargo build --all-targets --features dev"
      cargo build --all-targets --features dev
      echo ">>> [lele:build] done"
    '';
    "lele:clippy".exec = ''
      set -e
      echo ">>> [lele:clippy] cargo clippy --all-targets --features dev"
      cargo clippy --all-targets --features dev -- -D warnings
      echo ">>> [lele:clippy] cargo clippy --tests --features dev"
      cargo clippy --tests --features dev -- -D warnings
      echo ">>> [lele:clippy] done"
    '';
    "lele:fmt".exec = ''
      set -e
      echo ">>> [lele:fmt] cargo fmt -- --check"
      cargo fmt -- --check
      echo ">>> [lele:fmt] done"
    '';
    "lele:nextest".exec = ''
      set -e
      echo ">>> [lele:nextest] cargo nextest run --all-targets --features dev"
      cargo nextest run --all-targets --features dev
      echo ">>> [lele:nextest] done"
    '';
    "lele:bacon-clippy".exec = ''
      set -e
      echo ">>> [lele:bacon-clippy] bacon --headless clippy --features dev"
      bacon --headless clippy --features dev -- -- -D warnings
      echo ">>> [lele:bacon-clippy] done"
    '';
    "lele:lint".exec = ''
      set -e
      echo ">>> [lele:lint] lele_lint"
      cargo run --manifest-path ../lele_lint/Cargo.toml
      echo ">>> [lele:lint] done"
    '';
    "lele:taxonomy_check".exec = ''
      set -e
      echo ">>> [lele:taxonomy_check] lele_function_taxonomy"
      cargo run --manifest-path ../lele_function_taxonomy/Cargo.toml --features rustc-private -- --manifest-path ./Cargo.toml
      echo ">>> [lele:taxonomy_check] done"
    '';
    "lele:clippy".after = ["lele:build"];
    "lele:fmt".after = ["lele:clippy"];
    "lele:bacon-clippy".after = ["lele:fmt"];
    "lele:lint".after = ["lele:bacon-clippy"];
    "lele:taxonomy_check".after = ["lele:lint"];
    "lele:verify".after = ["lele:taxonomy_check"];
    "freenet:run-local-mainnet".exec = ''
      set -e
      echo ">>> [freenet:run-local-mainnet] cargo nextest run --test mainnet_local --features dev"
      cargo nextest run --test mainnet_local --features dev --run-ignored all -- --nocapture
    '';
    "freenet:run-cross-os".exec = ''
      set -e
      echo ">>> [freenet:run-cross-os] cargo nextest run --test mainnet_cross --features dev"
      cargo nextest run --test mainnet_cross --features dev --run-ignored all -- --nocapture
    '';
    "devenv:enterTest".after = [ "lele:verify" ];
  };

  git-hooks.hooks = {
    rustfmt.enable = true;
  };
}
