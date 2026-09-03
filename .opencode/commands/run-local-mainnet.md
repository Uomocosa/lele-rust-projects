---
description: "Run the local-mainnet test for any crate that provides it (prefers per-crate devenv task freenet:run-local-mainnet / freenet:run-cross-os). Generic over freenet_example and Bevy automation crates."
---

You are running the **programmatic local-mainnet test** — you do NOT spawn instances or drive windows by hand. The automation builds the binary, launches real separate instances that each join the **public Freenet mainnet independently** (no `--freenet-local`, no `--freenet-gateway`) on a shared throwaway `--contract-params`, waits for mutual convergence, records video (with wakeup to avoid screensaver) and sends MP4/report to Telegram. Your job is to invoke the right entry point for the requested crate, watch it, and report.

## Phase 0 — Pick the target project (explicit crate required — no default)

`$ARGUMENTS` must contain a crate name as the first token. If absent or unknown, fail fast and print:

```
usage: /run-local-mainnet <crate> [cross]
known crates: any crate with freenet:run-local-mainnet task or mainnet test/automation (e.g. freenet_example, freenet_libp2p_bevy_example)
aliases: example1/ex1, example2/ex2, example3/ex3, example4/ex4 map to Bevy crates; freenet_example_3 is an alias for freenet_example; prefer full crate name.
For crates with tests/e2e_local/mainnet.rs the run always spawns 3 instances and always sends video via Telegram — no flags. Second arg "cross" selects the cross-host variant (freenet:run-cross-os) where available.
```

Resolve generically (do not hardcode a single crate):

1. Normalize first token: `ex1`/`example1` → `freenet_libp2p_bevy_example_1`, `ex2`→`freenet_libp2p_bevy_example_2`, `ex3`→`freenet_libp2p_bevy_example_3`, `ex4`→`freenet_libp2p_bevy_example_4`, `freenet_example_3`→`freenet_example` (deprecated alias), otherwise use token as-is.
2. If token missing or `<crate>/Cargo.toml` not found, discover candidates: `ls -d freenet_*/ 2>/dev/null`, plus `grep -l "freenet:run-local-mainnet\|mainnet" freenet_*/devenv.nix freenet_*/Cargo.toml` and list them in the usage error, then exit.
3. Determine entry point by inspecting the crate (prefer devenv):
   - If `<crate>/devenv.nix` defines `freenet:run-local-mainnet` (and `freenet:run-cross-os` for `cross` arg) → use that task. This is the preferred path for any crate (e.g. `freenet_example` → `tests/e2e_local/mainnet.rs` / `tests/e2e_cross_os/mainnet.rs`, both `#[ignore]`, always 3 instances + always video/Telegram).
   - Else if `<crate>/tests/e2e_local/mainnet.rs` exists → raw fallback `cargo nextest run --test mainnet_local --run-ignored all -- --nocapture` (cross → `mainnet_cross`).
   - Else if crate is a Bevy workspace with `mainnet_automation*` member (e.g. `freenet_libp2p_bevy_example/mainnet_automation_4`) → `cargo run -p mainnet-automation-4` (resolve binary name from `[[bin]]` in that member).
   - Else → fail with `no local-mainnet entry point found for <crate>` + discovered candidates.

Examples (not exhaustive):

| crate | project dir | entry point (preferred → fallback) | game binary |
|-------|-------------|-------------------------------------|-------------|
| `freenet_example` (`freenet_example_3` alias) | `freenet_example` | `tests/e2e_local/mainnet.rs` via `devenv tasks run freenet:run-local-mainnet` / cross via `freenet:run-cross-os` | `freenet-example` |
| `freenet_libp2p_bevy_example` (ex4) | `freenet_libp2p_bevy_example` | `mainnet_automation_4` via `cargo run -p mainnet-automation-4` | `freenet-libp2p-bevy-example` |

Any crate that adds `freenet:run-local-mainnet` to its `devenv.nix:tasks` automatically becomes runnable via the same command — no command update needed.

## Phase 1 — Run the automation

Generic invocation (prefer devenv when defined):

```
# local variant (default):
cd <crate> && devenv tasks run freenet:run-local-mainnet
# cross-host variant (when $2 == "cross"):
cd <crate> && devenv tasks run freenet:run-cross-os
```

Tasks are `cargo nextest run --test mainnet_local --run-ignored all -- --nocapture` (or `mainnet_cross`), with `--nocapture` so logs stream live. Raw fallback when devenv is absent:

```
cd <crate> && CARGO_TARGET_DIR=/tmp/frt-build cargo nextest run --test mainnet_local --run-ignored all -- --nocapture
# cross: --test mainnet_cross
```

For Bevy automation crates (no devenv task):

```
cd <project_dir>
CARGO_TARGET_DIR=/tmp/frt-build cargo run -p <automation-crate> -- <remaining args>
```

- Do NOT run automation by hand phase-by-phase — it is one binary/test.
- `CARGO_TARGET_DIR=/tmp/frt-build` is mandatory only for raw `cargo …` (devenv sets `env.CARGO_TARGET_DIR` via `devenv.nix`).
- Automation prints `binary: ...`, `launched N instances`, convergence, video path, Telegram `send_video`/`send_text` lines.
- Drop guard / test teardown kills instances even on error.

## Phase 2 — Report
Print a concise report:
- Target crate + entry point invoked, exact command used (devenv task vs raw fallback vs cargo run).
- Automation summary: run-dir, contract params, instance count (always 3 for freenet_example-style), convergence verdict, pass/fail, error signatures.
- Whether Telegram delivery happened (`send_video` / `send_text` lines).
- Cleanup confirmation: `pgrep -af <game binary>` (`freenet-example`, `freenet-libp2p-bevy-example{,-*}`) — `pkill -f` if any remain.

## Hard rules
- Never spawn game instances or drive windows by hand — automation does.
- Never pass `--freenet-local` / `--freenet-gateway` (test is mainnet).
- No default target — crate argument is mandatory. Prefer `devenv tasks run freenet:run-local-mainnet` (and `freenet:run-cross-os` for cross-host) when the task exists.
- Always `devenv tasks run …` when `devenv.nix` defines it; raw `cargo …` is fallback only.
- Always `CARGO_TARGET_DIR=/tmp/frt-build` for raw `cargo` (devenv already sets it).
- Never end with game processes still alive (Phase 2 cleanup mandatory, even on failure).
