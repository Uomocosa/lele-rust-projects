{ pkgs, lib, config, inputs, ... }: {
  stdenv = pkgs.gccStdenv;

  languages.rust = {
    enable = true;
    channel = "nightly";
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
    wayland
    alsa-lib
    udev
    libxkbcommon
    mesa
    vulkan-loader
    xorg.libX11
    xorg.libXext
    xorg.libXrandr
    xorg.libXcursor
    xorg.libXi
    (if pkgs ? ffmpeg-full then pkgs.ffmpeg-full else ffmpeg)
    xorg.xdpyinfo
    xterm
    wmctrl
    xdotool
  ];

  env.CARGO_TARGET_DIR = "/tmp/frt-build";
  # Cold `--all-targets --all-features` build on 14 GB RAM: 8 jobs peak 13.3 GB / 4m16s,
  # 6 jobs peak 11.8 GB / 4m41s. Trade ~25 s for ~1.5 GB of headroom.
  env.CARGO_BUILD_JOBS = "6";
  env.C_INCLUDE_PATH = "${pkgs.glibc.dev}/include:${pkgs.linuxHeaders}/include";
  env.CFLAGS = "-I${pkgs.glibc.dev}/include -Wno-error";
  env.CPPFLAGS = "-I${pkgs.glibc.dev}/include -Wno-error";
  env.VK_ICD_FILENAMES = "${pkgs.mesa}/share/vulkan/icd.d/lvp_icd.x86_64.json";
  env.LD_LIBRARY_PATH = lib.makeLibraryPath [
    pkgs.libxkbcommon
    pkgs.mesa
    pkgs.wayland
    pkgs.alsa-lib
    pkgs.udev
    pkgs.vulkan-loader
    pkgs.xorg.libX11
    pkgs.xorg.libXext
    pkgs.xorg.libXrandr
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
  ];

  tasks = {
    "lele:build" = { exec = "cargo build --all-targets --all-features"; showOutput = true; };
    "lele:clippy" = { exec = "cargo clippy --all-targets --all-features -- -D warnings"; showOutput = true; };
    "lele:fmt" = { exec = "cargo fmt -- --check"; showOutput = true; };
    "lele:nextest" = { exec = "cargo nextest run --all-targets --all-features"; showOutput = true; };
    "lele:lint" = { exec = "CARGO_TARGET_DIR=$HOME/.cache/cargo-target/lele_lint cargo run --manifest-path ../lele_lint/Cargo.toml"; showOutput = true; };
    "lele:taxonomy_check" = { exec = "bash -c 'grep -q require lele.toml 2>/dev/null || exit 0; d=$HOME/.cache/cargo-target/lele_taxonomy_tool; devenv --from path:../lele_function_taxonomy shell -- bash -c \"CARGO_TARGET_DIR=$d cargo build --quiet --manifest-path ../lele_function_taxonomy/Cargo.toml --bins\" && LELE_TAXONOMY_DRIVER=$d/debug/lele-taxonomy-driver $d/debug/lele-function-taxonomy --manifest-path ./Cargo.toml'"; showOutput = true; };
    "freenet:contract-harness" = { exec = "cargo test --manifest-path ../freenet_contract_harness/Cargo.toml -- --nocapture"; showOutput = true; };
    "freenet:run-local-mainnet" = { exec = "cargo nextest run --test mainnet_local --all-features --run-ignored all -- --nocapture"; showOutput = true; };
    "freenet:run-cross-os" = { exec = "cargo nextest run --test mainnet_cross --all-features --run-ignored all -- --nocapture"; showOutput = true; };
    "lobby:e2e-discovery" = { exec = "cargo nextest run --test lobby_room_discovery --all-features --run-ignored all -- --nocapture"; showOutput = true; };
    "lobby:e2e-join" = { exec = "cargo nextest run --test lobby_room_join --all-features --run-ignored all -- --nocapture"; showOutput = true; };
    "lele:docs:check" = { exec = "cargo run --manifest-path ../lele_code_viewer/Cargo.toml -- check"; showOutput = true; };
    "lele:docs:serve" = { exec = "cargo run --manifest-path ../lele_code_viewer/Cargo.toml -- serve"; showOutput = true; };
    "lele:docs:export" = { exec = "cargo run --manifest-path ../lele_code_viewer/Cargo.toml -- export --out target/docs-site"; showOutput = true; };
  };

  git-hooks.hooks = {
    lele-clippy = {
      enable = true;
      name = "clippy (freenet_libp2p_bevy_plugin)";
      entry = "bash -c 'cd freenet_libp2p_bevy_plugin && devenv tasks run lele:clippy 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-fmt = {
      enable = true;
      name = "fmt (freenet_libp2p_bevy_plugin)";
      entry = "bash -c 'cd freenet_libp2p_bevy_plugin && devenv tasks run lele:fmt 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-lint = {
      enable = true;
      name = "lele_lint (freenet_libp2p_bevy_plugin)";
      entry = "bash -c 'cd freenet_libp2p_bevy_plugin && devenv tasks run lele:lint 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-taxonomy = {
      enable = true;
      name = "taxonomy_check (freenet_libp2p_bevy_plugin)";
      entry = "bash -c 'cd freenet_libp2p_bevy_plugin && devenv tasks run lele:taxonomy_check 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    freenet-contract-harness = {
      enable = true;
      name = "contract-harness (freenet_libp2p_bevy_plugin)";
      entry = "bash -c 'cd freenet_libp2p_bevy_plugin && devenv tasks run freenet:contract-harness 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-docs = {
      enable = true;
      name = "lele_code_viewer (freenet_libp2p_bevy_plugin)";
      entry = "bash -c 'cd freenet_libp2p_bevy_plugin && devenv tasks run lele:docs:check 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
  };
}
