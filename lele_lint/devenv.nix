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
    "lele:verify".exec = ''
      cargo build --all-targets
      cargo clippy -- -D warnings
      cargo fmt -- --check
      cargo nextest run --all-targets
      bacon --headless clippy -- -- -D warnings
      cargo run --manifest-path ../lele_lint/Cargo.toml
    '';
    "lele:nextest".exec = "cargo nextest run --all-targets";
    "lele:bacon-clippy".exec = "bacon --headless clippy -- -- -D warnings";
    "lele:lint".exec = "cargo run --manifest-path ../lele_lint/Cargo.toml";
    "devenv:enterTest".after = [ "lele:verify" ];
  };

  git-hooks.hooks = {
    rustfmt.enable = true;
  };
}
