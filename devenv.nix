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

  tasks."lele:enforce-config" = {
    exec = "cargo run --manifest-path lele_enforce_config/Cargo.toml 2>&1";
    showOutput = true;
  };

  git-hooks.hooks.lele-enforce-config = {
    enable = true;
    name = "lele_enforce_config";
    entry = "devenv tasks run lele:enforce-config 2>&1";
    pass_filenames = false;
    always_run = true;
  };
}
