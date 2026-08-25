---
description: "Run the proper local-mainnet test for the target project by executing its automation crate (not by hand). Default target: freenet_libp2p_bevy_example_4. Pass 'example1'/'example_1'/'ex1' or 'example2'/'example_2'/'ex2' or 'example3'/'example_3'/'ex3' to target example_1, example_2, or example_3 instead. The crate builds the game, launches N real independent mainnet instances (no --freenet-local/gateway, shared throwaway --contract-params), waits for mutual convergence, tiles + drives the windows, records a screen video, and sends the MP4 + a report to Telegram — all programmatically. Remaining args (instance count, 'release', '--no-video', '--no-telegram', '--timeout') are passed through to the binary."
---

You are running the **programmatic local-mainnet test** for the target project — you do NOT
spawn instances or drive windows by hand. A dedicated automation crate does all of it: it builds
the game, launches N real, separate instances that each join the **public Freenet mainnet
independently** (no `--freenet-local`, no `--freenet-gateway`) on a shared throwaway
`--contract-params`, waits for mutual convergence, tiles + moves the windows with `x11rb`,
records a screen video, then sends the MP4 and a final report to Telegram via the bot. Your job is
to pick the target, invoke that crate with the right args, watch it, and report the verdict.

## Phase 0 — Pick the target project and its automation crate
Parse `$ARGUMENTS`:
- Contains `example1`, `example_1`, or `ex1` → target **example_1** (crate `mainnet-automation`).
- Contains `example2`, `example_2`, or `ex2` → target **example_2** (crate `mainnet-automation-2`).
- Contains `example3`, `example_3`, or `ex3` → target **example_3** (crate `mainnet-automation-3`).
- Otherwise → target **example_4** (crate `mainnet-automation-4`). This is the default.
  (`example4`, `example_4`, and `ex4` are accepted explicitly for example_4.)
- The remaining tokens are passed straight through to the automation binary: a leading integer
  is the instance count, `release` selects a release build, and `--no-video`, `--no-telegram`,
  `--timeout N` are flags. Build the arg list after removing the target-selection token.

Resolve per target:
| target | project dir | crate | game binary |
|--------|-------------|-------|-------------|
| example_1 | `freenet_libp2p_bevy_example_1` | `mainnet-automation` | `freenet-libp2p-bevy-example-1` |
| example_2 | `freenet_libp2p_bevy_example_2` | `mainnet-automation-2` | `freenet-libp2p-bevy-example-2` |
| example_3 | `freenet_libp2p_bevy_example_3` | `mainnet-automation-3` | `freenet-libp2p-bevy-example-3` |
| example_4 | `freenet_libp2p_bevy_example_4` | `mainnet-automation-4` | `freenet-libp2p-bevy-example-4` |

## Phase 1 — Run the automation crate
```
cd <project_dir>
CARGO_TARGET_DIR=/tmp/frt-build cargo run -p <crate> -- <remaining args>
```
- Do NOT run the automation by hand phase-by-phase — it is one binary.
- `CARGO_TARGET_DIR=/tmp/frt-build` is mandatory (the workspace root is under a space-containing
  path and `build_game` builds the game from within the crate; the env var propagates so the
  resolved binary lands where `cargo metadata` reports).
- The automation prints progress to stdout/stderr: `binary: ...`, `run-dir: ...`, `contract:
  ...`, `launched N instances`, `all N instances mutually converged`, the tiling layout, and the
  final report; it exits non-zero on an assertion failure (`run did not pass: moved=...`).
- The automation itself sends the video + text report to Telegram when creds are present.
- It tears the instances down itself (a drop guard kills them even on error).

## Phase 2 — Report
Print a concise report to the user:
- Target project + crate invoked, and the exact command used.
- The automation's own summary: run-dir, contract params, instance count, convergence verdict,
  moved/pass-fail line, error signatures (if any).
- Whether Telegram delivery happened (look for the `send_video` / `send_text` step lines).
- Cleanup confirmation: after the run, verify no leftover game processes remain with
  `pgrep -af <game binary>` using the game binary column of the Phase 0 table
  (`freenet-libp2p-bevy-example-{1,2,3,4}`) — the automation should have killed them;
  if any remain, `pkill -f` them and say so.

## Hard rules
- Never spawn the game instances or drive windows by hand — that is what the cleanup crate does.
- Never pass `--freenet-local` / `--freenet-gateway` to the automation (the test is mainnet).
- Default target is example_4; switch only when `$ARGUMENTS` names example_1, example_2, or example_3.
- Always `CARGO_TARGET_DIR=/tmp/frt-build`.
- Never end with game processes still alive (Phase 2 cleanup confirmation is mandatory, even on failure).
