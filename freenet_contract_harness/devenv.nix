{ pkgs, lib, config, inputs, ... }: {
  languages.rust = {
    enable = true;
    channel = "stable";
    components = [ "rustc" "cargo" "clippy" "rustfmt" ];
  };

  packages = with pkgs; [
    bacon
    cargo-nextest
  ];

  env.CARGO_TARGET_DIR = "/tmp/frt-build";

  tasks = {
    "lele:build".exec = ''
      set -e
      echo ">>> [lele:build] cargo build --all-targets"
      cargo build --all-targets
      echo ">>> [lele:build] done"
    '';
    "lele:clippy".exec = ''
      set -e
      echo ">>> [lele:clippy] cargo clippy --all-targets -- -D warnings"
      cargo clippy --all-targets -- -D warnings
      echo ">>> [lele:clippy] cargo clippy --tests -- -D warnings"
      cargo clippy --tests -- -D warnings
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
      echo ">>> [lele:nextest] cargo nextest run --all-targets"
      cargo nextest run --all-targets
      echo ">>> [lele:nextest] done"
    '';
    "lele:bacon-clippy".exec = ''
      set -e
      echo ">>> [lele:bacon-clippy] bacon --headless clippy -- -- -D warnings"
      bacon --headless clippy -- -- -D warnings
      echo ">>> [lele:bacon-clippy] done"
    '';
    "lele:lint".exec = ''
      set -e
      echo ">>> [lele:lint] lele_lint"
      cargo run --manifest-path ../lele_lint/Cargo.toml
      echo ">>> [lele:lint] done"
    '';
    "lele:clippy".after = ["lele:build"];
    "lele:fmt".after = ["lele:clippy"];
    "lele:bacon-clippy".after = ["lele:fmt"];
    "lele:lint".after = ["lele:bacon-clippy"];
    "lele:verify".after = ["lele:lint"];
    "devenv:enterTest".after = ["lele:verify"];
  };

  git-hooks.hooks = {
    rustfmt.enable = true;
  };
}
