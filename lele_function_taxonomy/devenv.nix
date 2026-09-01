{ pkgs, lib, config, inputs, ... }: {
  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
  };

  packages = with pkgs; [
    bacon
    cargo-nextest
  ];

  env.CARGO_TARGET_DIR = "/tmp/frt-build";

  tasks = {
    "lele:taxonomy".exec = ''
      cargo run -p lele_function_taxonomy --features rustc-private -- --manifest-path ./Cargo.toml
    '';
    "lele:verify".exec = ''
      cargo build --features rustc-private --all-targets
      cargo clippy --features rustc-private -- -D warnings
      cargo fmt -- --check
      cargo nextest run --features rustc-private --all-targets
      bacon --headless clippy --features rustc-private -- -- -D warnings
      cargo run --features rustc-private -- --manifest-path ./Cargo.toml
    '';
    "lele:nextest".exec = "cargo nextest run --features rustc-private --all-targets";
    "lele:bacon-clippy".exec = "bacon --headless clippy --features rustc-private -- -- -D warnings";
    "devenv:enterTest".after = [ "lele:verify" ];
  };

  git-hooks.hooks.rustfmt.enable = true;
}
