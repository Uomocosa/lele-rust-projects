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
  env.CARGO_BUILD_JOBS = "6";

  tasks = {
    "lele:build" = { exec = "cargo build --all-targets --all-features"; showOutput = true; };
    "lele:clippy" = { exec = "cargo clippy --all-targets --all-features -- -D warnings"; showOutput = true; };
    "lele:fmt" = { exec = "cargo fmt -- --check"; showOutput = true; };
    "lele:nextest" = { exec = "cargo nextest run --all-targets --all-features"; showOutput = true; };
    "lele:bench" = { exec = "cargo bench --all-features"; showOutput = true; };
    "lele:lint" = { exec = "CARGO_TARGET_DIR=$HOME/.cache/cargo-target/lele_lint cargo run --manifest-path ../lele_lint/Cargo.toml"; showOutput = true; };
    "lele:taxonomy_check" = { exec = "cargo run --manifest-path ../lele_function_taxonomy/Cargo.toml --bin lele-function-taxonomy -- --manifest-path ./Cargo.toml"; showOutput = true; };
    "lele:docs:check" = { exec = "cargo run -- check"; showOutput = true; };
    "lele:docs:serve" = { exec = "cargo run -- serve"; showOutput = true; };
    "lele:docs:export" = { exec = "cargo run -- export --out target/docs-site"; showOutput = true; };

    # Systemd user service `lele-code-viewer` (unit: ~/.config/systemd/user/lele-code-viewer.service,
    # created once by hand; runs ~/.local/bin/lele-code-viewer serve --watch on :8787).

    # Rebuild in release mode, replace the installed binary, restart the service so it serves the new code.
    "lele:service:update" = {
      exec = ''
        set -e
        cargo build --release
        install -Dm755 "$CARGO_TARGET_DIR/release/lele-code-viewer" "$HOME/.local/bin/lele-code-viewer"
        systemctl --user restart lele-code-viewer.service
        echo "lele-code-viewer.service: $(systemctl --user is-active lele-code-viewer.service)"
      '';
      showOutput = true;
    };
    # Show whether the service is running, since when, its pid and the last few log lines.
    "lele:service:status" = { exec = "systemctl --user status lele-code-viewer.service --no-pager"; showOutput = true; };
    # Print the last 200 log lines of the service and exit (safe for agents and scripts).
    "lele:service:logs" = { exec = "journalctl --user -u lele-code-viewer.service -n 200 --no-pager"; showOutput = true; };
    # Stream the service logs live until Ctrl+C (interactive, user-only: never returns on its own).
    "lele:service:logs-follow" = { exec = "journalctl --user -u lele-code-viewer.service -f"; showOutput = true; };
  };

  git-hooks.hooks = {
    lele-clippy = {
      enable = true;
      name = "clippy (lele_code_viewer)";
      entry = "bash -c 'cd lele_code_viewer && devenv tasks run lele:clippy 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-fmt = {
      enable = true;
      name = "fmt (lele_code_viewer)";
      entry = "bash -c 'cd lele_code_viewer && devenv tasks run lele:fmt 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-lint = {
      enable = true;
      name = "lele_lint (lele_code_viewer)";
      entry = "bash -c 'cd lele_code_viewer && devenv tasks run lele:lint 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-docs = {
      enable = true;
      name = "lele_code_viewer (lele_code_viewer)";
      entry = "bash -c 'cd lele_code_viewer && devenv tasks run lele:docs:check 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-taxonomy = {
      enable = true;
      name = "taxonomy_check (lele_code_viewer)";
      entry = "bash -c 'cd lele_code_viewer && devenv tasks run lele:taxonomy_check 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
  };
}
