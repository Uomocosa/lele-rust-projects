---
description: "Launch N real, separate OS processes of the freenet_libp2p_bevy_example_1 binary on THIS machine (default N=3), wired into one isolated local Freenet network (no mainnet), each with its own identity/log directory, so a human or agent can manually observe multiplayer behavior (roster sync, box spawn positions, movement sync) instead of relying on the single-process test harness. Sends window screenshots, a short screen recording, and a final summary to Telegram via deskctrl. Optional numeric argument sets the instance count; 'release' anywhere in the argument switches to a release build."
---

You are launching `$ARGUMENTS`-configured local instances of the `freenet_libp2p_bevy_example_1` game binary as **real, independent processes** on this machine — not the `ProductionGameApp`/`testing` in-process harness used by `integration_tests`/`e2e_tests`/`cross_os_tests`. Use this when a bug report needs eyes-on confirmation across multiple simultaneous game windows (e.g. "do newly spawned boxes land in free space now") rather than an automated assertion.

## Why this isn't the same as the e2e/integration tests

`integration_tests/local_two_node_production_sync.rs` and `e2e_tests/e2e_three_node_production_sync.rs` do the equivalent thing **in one process**, via `testing::ProductionGameApp` — multiple embedded Bevy `App`s ticked manually in a loop, no real windows, no real wall-clock pacing. They're the right tool for CI assertions. They are the **wrong** tool when you need to actually look at N game windows, or reproduce a bug that depends on real windowing/input/frame-pacing. This command drives the actual compiled binary, N times, exactly like N humans launching the game.

## Phase 0 — Determine instance count and build mode
Parse `$ARGUMENTS`:
- First integer found → instance count `N` (default `3` if none given).
- Contains `release` → `MODE=release`, else `MODE=dev`.

## Phase 1 — Build once
```
cd freenet_libp2p_bevy_example_1
cargo build --workspace [--release if MODE=release]
```
Resolve `BIN` = `target/debug/freenet-libp2p-bevy-example-1` (or `target/release/...` for release), matching the crate's `[[bin]]` name in `Cargo.toml`. Do not use `cargo run` to launch instances — the binary must be spawned directly so its OS pid is the one deskctrl's `list_windows`/`screenshot` can match (see `.opencode/skills/deskctrl/SKILL.md`).

## Phase 2 — Prepare the run directory (the logging story)
```
RUN_DIR=freenet_libp2p_bevy_example_1/.local-run/$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$RUN_DIR"
```
For each instance `i` in `1..N`, create `"$RUN_DIR/instance-$i"` with a dedicated `identity/` subdir (passed to `--identity-dir`) and an `app.log` file. Keeping identity dirs separate is required — `p2p::load_or_create_keypair` derives the player id from the identity dir's keypair, so instances sharing one dir would collide.

## Phase 3 — Launch instance 1 as the local gateway
Instance 1 is the isolated local gateway (`--freenet-local`): it does not touch the public Freenet mainnet, and every other instance dials it directly. Spawn via deskctrl `spawn_process`, wrapped so redirection survives while the OS pid still resolves to the game binary (`bash -c "exec ..."` replaces the bash process image in place — no extra pid layer):
```
bash -c "exec env RUST_LOG=warn,roster=info,p2p=info RUST_BACKTRACE=1 $BIN --identity-dir $RUN_DIR/instance-1/identity --freenet-local > $RUN_DIR/instance-1/app.log 2>&1"
```
Record the returned deskctrl PID and `os_pid`.

## Phase 4 — Extract the gateway address from instance 1's log
Poll `$RUN_DIR/instance-1/app.log` (Bash `grep`, or Monitor's until-loop) for the line emitted by `src/roster/start_embedded_node.rs:93-98`:
```
embedded node ready; dial as 127.0.0.1:<public-port>,<pubkey-hex>  public_key_hex=... public_port=...
```
Parse `public_port` and `public_key_hex` from that line's fields and build:
```
GATEWAY="127.0.0.1:${public_port},${public_key_hex}"
```
If this line hasn't appeared within ~90s, stop and report instance 1's log tail — the embedded node failed to reach `wait_ready` (see `start_embedded_node.rs:88-91`).

## Phase 5 — Launch instances 2..N as clients of the gateway
For each remaining instance, same `bash -c "exec env ... > log 2>&1"` pattern, but with `--freenet-gateway "$GATEWAY"` instead of `--freenet-local`:
```
bash -c "exec env RUST_LOG=warn,roster=info,p2p=info RUST_BACKTRACE=1 $BIN --identity-dir $RUN_DIR/instance-$i/identity --freenet-gateway $GATEWAY > $RUN_DIR/instance-$i/app.log 2>&1"
```

## Phase 6 — Confirm convergence from the logs
For every instance's `app.log`, poll for evidence of a successful roster join (targets are all `target: "roster"`, `warn,roster=info,p2p=info` is on):
- `roster GetResponse` (found the existing/seeded roster, `src/roster/setup_contract.rs:101-106`)
- `merging own entry, sending roster Update` (`setup_contract.rs:117-121`) or, for instance 1 only, `sending roster Put` if it was first to seed
- Any `error exited with error`, `panicked`, or `update confirmation timed out` / `timeout after 30s` lines are the failure signatures to grep for first when something's wrong.

## Phase 7 — Visual confirmation, sent to Telegram
Use deskctrl `list_windows` and match each instance's `os_pid` (from Phase 3/5) to a window (never `title` — two instances have identical titles). This phase is not optional when deskctrl is available; it's the evidence attached to the run:
1. `screenshot` each instance's window by `window_id` — useful to eyeball box spawn positions (the bug this kind of run is often used to check). `send_to_telegram` defaults to `true`, so each photo is posted as it's taken; give each a `caption` naming the instance (e.g. `"instance-2 window"`).
2. `record_video` (no `window_id`/`pid`/`title` → whole screen, so all N windows are visible together) for ~15-30s while boxes are moving/roster is converging, then `record_video stop:true` with a `summary` describing what it shows — this posts the MP4 to Telegram automatically.

## Phase 8 — Tear every instance down
**Always kill the instances at the end — never leave them running.** Each one holds an embedded Freenet node and sockets; leftovers from an earlier run collide with the next one's gateway wiring and quietly skew its results.

Do this *after* Phase 7 has captured its screenshots and video (those artifacts are the durable record — the live windows are not), and do it even when the run failed or ended early:
1. deskctrl `kill_process` for each recorded instance PID.
2. Then `pkill -f freenet-libp2p-bevy-example-1` from Bash to catch anything deskctrl did not own.
3. Confirm with `pgrep -af freenet-libp2p-bevy-example-1` and report the result. Nothing should remain.

The logs under `$RUN_DIR` survive the kill — that is where the evidence lives. If the user asks to keep the windows up for hands-on poking, they will say so; default to tearing down.

## Phase 9 — Report
Compose the report below, print it to the user, and also send it via `send_to_telegram` (`text` = the same content) so it's the final message in the same Telegram thread as the screenshots/video from Phase 7:
- `RUN_DIR` and each instance's `app.log` path (their permanent investigation artifact).
- Each instance's deskctrl PID / OS pid / window id.
- The `GATEWAY` string used.
- A one-line grep cheat-sheet for later debugging: `grep -E "error|panic|timed out|NotFound" "$RUN_DIR"/instance-*/app.log`.
- Confirmation that all instances were killed and `pgrep` came back empty.

## Hard rules
- Never launch via `cargo run` — direct binary spawn only, so os_pid tracking works.
- Always give every instance its own `--identity-dir`.
- Instance 1 is always `--freenet-local`; every other instance always dials it via `--freenet-gateway`, never `--freenet-local` itself and never the public mainnet — this stays fully isolated from production.
- Never delete or overwrite a previous `$RUN_DIR` — always timestamp a fresh one.
- Never end the run with game processes still alive — Phase 8 is mandatory, including on the failure path.
