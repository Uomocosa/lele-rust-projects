{ pkgs, lib, config, inputs, ... }: {
  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [ "rustc" "cargo" "clippy" "rustfmt" ];
  };

  packages = with pkgs; [
    cargo-nextest
    clang
    pkg-config
    glibc.dev
    linuxHeaders
    wayland
    alsa-lib
    udev
    libxkbcommon
    xorg.libX11
    xorg.libXext
    xorg.libXrandr
    xorg.libXcursor
    xorg.libXi
    gnumake
    mesa
    vulkan-loader
    (if pkgs ? ffmpeg-full then pkgs.ffmpeg-full else ffmpeg)
  ];

  env.CARGO_TARGET_DIR = "/tmp/frt-build";
  env.CARGO_BUILD_JOBS = "6";
  env.VK_ICD_FILENAMES = "${pkgs.mesa}/share/vulkan/icd.d/lvp_icd.x86_64.json";
  env.C_INCLUDE_PATH = "${pkgs.glibc.dev}/include:${pkgs.linuxHeaders}/include";
  env.CFLAGS = "-I${pkgs.glibc.dev}/include -Wno-error";
  env.CPPFLAGS = "-I${pkgs.glibc.dev}/include -Wno-error";
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
    "lele:bevy-lint" = { exec = "cargo run --manifest-path ../lele_bevy_lint/Cargo.toml"; showOutput = true; };
    "lele:taxonomy_check" = { exec = "bash -c 'grep -q require lele.toml 2>/dev/null || exit 0; d=$HOME/.cache/cargo-target/lele_taxonomy_tool; cfg=.pre-commit-config.yaml; hook=$(git rev-parse --git-path hooks/pre-commit); if [ -e $cfg ]; then cp -a $cfg $cfg.lelebak; fi; if [ -e $hook ]; then cp -a $hook $hook.lelebak; fi; devenv --from path:../lele_function_taxonomy shell -- bash -c \"CARGO_TARGET_DIR=$d cargo build --quiet --manifest-path ../lele_function_taxonomy/Cargo.toml --bins\"; rc=$?; if [ -e $cfg.lelebak ]; then mv -f $cfg.lelebak $cfg; fi; if [ -e $hook.lelebak ]; then mv -f $hook.lelebak $hook; fi; [ $rc -eq 0 ] || exit $rc; LELE_TAXONOMY_DRIVER=$d/debug/lele-taxonomy-driver $d/debug/lele-function-taxonomy --manifest-path ./Cargo.toml'"; showOutput = true; };
    "ui:previews" = { exec = "cargo nextest run --all-features -- --nocapture"; showOutput = true; };
  };

  git-hooks.hooks = {
    lele-clippy = {
      enable = true;
      name = "clippy (lele_bevy_preview)";
      entry = "bash -c 'cd lele_bevy_preview && devenv tasks run lele:clippy 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-fmt = {
      enable = true;
      name = "fmt (lele_bevy_preview)";
      entry = "bash -c 'cd lele_bevy_preview && devenv tasks run lele:fmt 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
    lele-lint = {
      enable = true;
      name = "lele_lint (lele_bevy_preview)";
      entry = "bash -c 'cd lele_bevy_preview && devenv tasks run lele:lint 2>&1'";
      pass_filenames = false;
      always_run = true;
    };
  };
}