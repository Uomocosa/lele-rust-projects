{ pkgs, lib, config, inputs, ... }: {
  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
  };

  packages = with pkgs; [
    cargo-nextest
  ];

  env.CARGO_TARGET_DIR = "/tmp/frt-build";
  env.CARGO_BUILD_JOBS = "6";

  tasks = {
    "lele:build" = { exec = "cargo build --all-targets --all-features"; showOutput = true; };
    "lele:clippy" = { exec = "cargo clippy --all-targets --all-features -- -D warnings"; showOutput = true; };
    "lele:fmt" = { exec = "cargo fmt -- --check"; showOutput = true; };
    "lele:nextest" = { exec = "cargo nextest run --all-targets --all-features"; showOutput = true; };
    "lele:taxonomy" = { exec = "cargo run --bin lele-function-taxonomy -- --manifest-path ./Cargo.toml"; showOutput = true; };
    "lele:lint" = { exec = "cargo run --bin lele-function-taxonomy -- --manifest-path ./Cargo.toml"; showOutput = true; };
    "lele:taxonomy_check" = { exec = "bash -c 'grep -q require lele.toml 2>/dev/null || exit 0; d=$HOME/.cache/cargo-target/lele_taxonomy_tool; cfg=.pre-commit-config.yaml; hook=$(git rev-parse --git-path hooks/pre-commit); if [ -e $cfg ]; then cp -a $cfg $cfg.lelebak; fi; if [ -e $hook ]; then cp -a $hook $hook.lelebak; fi; devenv --from path:../lele_function_taxonomy shell -- bash -c \"CARGO_TARGET_DIR=$d cargo build --quiet --manifest-path ../lele_function_taxonomy/Cargo.toml --bins\"; rc=$?; if [ -e $cfg.lelebak ]; then mv -f $cfg.lelebak $cfg; fi; if [ -e $hook.lelebak ]; then mv -f $hook.lelebak $hook; fi; [ $rc -eq 0 ] || exit $rc; LELE_TAXONOMY_DRIVER=$d/debug/lele-taxonomy-driver $d/debug/lele-function-taxonomy --manifest-path ./Cargo.toml'"; showOutput = true; };
  };

  git-hooks.hooks = {
    lele-clippy = {
      enable = true;
      name = "clippy (lele_function_taxonomy)";
      entry = "bash -c 'cd lele_function_taxonomy && devenv tasks run lele:clippy 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-fmt = {
      enable = true;
      name = "fmt (lele_function_taxonomy)";
      entry = "bash -c 'cd lele_function_taxonomy && devenv tasks run lele:fmt 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-lint = {
      enable = true;
      name = "lele_lint (lele_function_taxonomy)";
      entry = "bash -c 'cd lele_function_taxonomy && devenv tasks run lele:lint 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-taxonomy = {
      enable = true;
      name = "taxonomy_check (lele_function_taxonomy)";
      entry = "bash -c 'cd lele_function_taxonomy && devenv tasks run lele:taxonomy_check 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
  };
}
