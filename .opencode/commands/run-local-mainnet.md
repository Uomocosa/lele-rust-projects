---
description: "Run the local-mainnet test for an explicit crate (no default). For freenet_example_3 use devenv tasks freenet:run-local-mainnet / freenet:run-cross-os (tests/e2e_local/mainnet.rs + tests/e2e_cross_os/mainnet.rs, both #[ignore], always 3 instances + always video/Telegram); for Bevy examples use mainnet-automation-* crates."
---

You are running the **programmatic local-mainnet test** — you do NOT spawn instances or drive windows by hand. The automation builds the binary, launches real separate instances that each join the **public Freenet mainnet independently** (no `--freenet-local`, no `--freenet-gateway`) on a shared throwaway `--contract-params`, waits for mutual convergence, records video (with wakeup to avoid screensaver) and sends MP4/report to Telegram. Your job is to invoke the right entry point for the requested crate, watch it, and report.

## Phase 0 — Pick the target project (explicit crate required — no default)

`$ARGUMENTS` must contain a crate name as the first token. If absent or unknown, fail fast and print:

```
usage: /run-local-mainnet <crate>
known crates: freenet_example_3, freenet_example, freenet_example_2, freenet_libp2p_bevy_example_1, freenet_libp2p_bevy_example_2, freenet_libp2p_bevy_example_3, freenet_libp2p_bevy_example_4
aliases: example1/ex1, example2/ex2, example3/ex3, example4/ex4 map to the Bevy crates; prefer full crate name.
For freenet_example_3 the run always spawns 3 instances and always sends video via Telegram — no flags.
```

Resolve per target:

| target | project dir | entry point | game binary |
|--------|-------------|-------------|-------------|
| freenet_example_3 (local) | `freenet_example_3` | `tests/e2e_local/mainnet.rs` via `devenv tasks run freenet:run-local-mainnet` | `freenet-example-3` |
| freenet_example_3 (cross) | `freenet_example_3` | `tests/e2e_cross_os/mainnet.rs` via `devenv tasks run freenet:run-cross-os` | `freenet-example-3` |
| freenet_example | `freenet_example` | `e2e_mainnet` crate (legacy) | `freenet-example` |
| freenet_example_2 | `freenet_example_2` | `e2e_mainnet` crate | `freenet-example-2` |
| example_1 / freenet_libp2p_bevy_example_1 | `freenet_libp2p_bevy_example_1` | `mainnet-automation` | `freenet-libp2p-bevy-example-1` |
| example_2 / freenet_libp2p_bevy_example_2 | `freenet_libp2p_bevy_example_2` | `mainnet-automation-2` | `freenet-libp2p-bevy-example-2` |
| example_3 / freenet_libp2p_bevy_example_3 | `freenet_libp2p_bevy_example_3` | `mainnet-automation-3` | `freenet-libp2p-bevy-example-3` |
| example_4 / freenet_libp2p_bevy_example_4 | `freenet_libp2p_bevy_example_4` | `mainnet-automation-4` | `freenet-libp2p-bevy-example-4` |

For `freenet_example_3` the **default** is `tests/e2e_local/mainnet.rs` (always 3, always video/Telegram). Cross-host is `tests/e2e_cross_os/mainnet.rs`. Both are `#[ignore]` fixed-config tests, no CLI flags.

## Phase 1 — Run the automation

For `freenet_example_3` (preferred — per-crate devenv task, fixed config):

```
cd freenet_example_3
devenv tasks run freenet:run-local-mainnet
# cross-host:
devenv tasks run freenet:run-cross-os
```

Each task is `cargo nextest run --test mainnet_local --run-ignored all -- --nocapture` (or `mainnet_cross`), with `--nocapture` so logs stream live. Raw fallback when devenv is absent: `CARGO_TARGET_DIR=/tmp/frt-build cargo nextest run --test mainnet_local --run-ignored all -- --nocapture`.

For Bevy examples (binary crate):

```
cd <project_dir>
CARGO_TARGET_DIR=/tmp/frt-build cargo run -p <crate> -- <remaining args>
```

- Do NOT run automation by hand phase-by-phase — it is one binary/test.
- `CARGO_TARGET_DIR=/tmp/frt-build` is mandatory only for raw `cargo …` (devenv sets `env.CARGO_TARGET_DIR` via `devenv.nix`).
- Automation prints `binary: ...`, `launched 3 instances`, convergence, video path, Telegram `send_video`/`send_text` lines.
- Drop guard / test teardown kills instances even on error.

## Phase 2 — Report
Print a concise report:
- Target crate + entry point invoked, exact command used (devenv task).
- Automation summary: run-dir, contract params, instance count (always 3 for freenet_example_3), convergence verdict, pass/fail, error signatures.
- Whether Telegram delivery happened (`send_video` / `send_text` lines).
- Cleanup confirmation: `pgrep -af <game binary>` (`freenet-example-3`, `freenet-libp2p-bevy-example-{1,2,3,4}`) — `pkill -f` if any remain.

## Hard rules
- Never spawn game instances or drive windows by hand — automation does.
- Never pass `--freenet-local` / `--freenet-gateway` (test is mainnet).
- No default target — crate argument is mandatory. For `freenet_example_3` use `devenv tasks run freenet:run-local-mainnet` (and `freenet:run-cross-os` for cross-host).
- Always `devenv tasks run …` when `devenv.nix` defines it; raw `cargo …` is fallback only.
- Always `CARGO_TARGET_DIR=/tmp/frt-build` for raw `cargo` (devenv already sets it).
- Never end with game processes still alive (Phase 2 cleanup mandatory, even on failure).
