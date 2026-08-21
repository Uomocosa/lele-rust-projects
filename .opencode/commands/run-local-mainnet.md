---
description: "Launch N real, separate OS processes of the freenet_libp2p_bevy_example_1 binary on THIS (Linux) machine (default N=3), each discovering the others purely through the public Freenet mainnet (no --freenet-local, no --freenet-gateway) on a shared throwaway --contract-params value, so the exact production 'multiple users starting at once' race can be observed for real instead of via the run-local.md cross-machine CI pipeline or the hermetic run-local-instances.md gateway-dial pattern. Sends window screenshots, a short screen recording, and a final summary to Telegram via deskctrl. Optional numeric argument sets the instance count; 'release' anywhere in the argument switches to a release build."
---

You are launching `$ARGUMENTS`-configured local instances of the `freenet_libp2p_bevy_example_1` game binary as **real, independent processes**, all joining the **public Freenet mainnet independently**, with **zero coordination on how they find each other** — reproducing the real scenario of several players starting the app at once, but on just this Linux machine and on a throwaway contract instead of the real production one.

## Why this differs from the other two `run-local*` commands
- `run-local.md` drives the same idea across **two real machines** (Linux + Windows) via the self-hosted CI pipeline (GitHub Actions + `test-orchestrator` MCP).
- `run-local-instances.md` stays **fully hermetic**: instance 1 is an isolated `--freenet-local` gateway, every other instance dials it directly via `--freenet-gateway`. It never touches the mainnet and never exercises real peer discovery.
- **This command** is the missing middle case: real mainnet discovery (the actual production code path, `src/roster/connect_and_run.rs` with `local=false, gateway=None`), but only on this one machine, and isolated to a throwaway contract via `--contract-params` so it never touches the real shared production roster.

## Phase 0 — Determine instance count and build mode
Parse `$ARGUMENTS`: first integer found → instance count `N` (default `3`); contains `release` → `MODE=release`, else `MODE=dev`.

## Phase 1 — Build once
```
cd freenet_libp2p_bevy_example_1
cargo build --workspace [--release if MODE=release]
```
Resolve `BIN` via `cargo metadata --no-deps --format-version1 | jq -r .target_directory` (do **not** assume `target/debug` under the crate directory — this workspace overrides `target-dir` in `freenet_libp2p_bevy_example_1/.cargo/config.toml`, so check that file if `jq`/`cargo metadata` isn't available) joined with `debug/freenet-libp2p-bevy-example-1` or `release/...`. Sanity-check the resolved path with `$BIN --help` (prints all flags and exits) before spawning real instances. Never launch via `cargo run` — spawn the binary directly so deskctrl's `os_pid` matches the real game window (see `.opencode/skills/deskctrl/SKILL.md`).

## Phase 2 — Prepare the run directory and the shared throwaway contract
```
RUN_DIR=freenet_libp2p_bevy_example_1/.local-run/$(date -u +%Y%m%dT%H%M%SZ)-mainnet
mkdir -p "$RUN_DIR"
CONTRACT_PARAMS="local-mainnet-$(date +%s)-$RANDOM"
```
Create `"$RUN_DIR/instance-$i"` per instance with its own `identity/` subdir and `app.log`. `CONTRACT_PARAMS` is generated **once** and reused identically by every instance — same contract, unrelated to the real production one (which always uses empty params).

## Phase 3 — Launch all N instances with no local/gateway wiring
Every instance, including the first, uses the same pattern — **no** `--freenet-local`, **no** `--freenet-gateway`:
```
bash -c "exec env RUST_LOG=warn,roster=trace,p2p=debug,freenet_bevy=debug RUST_BACKTRACE=1 $BIN --identity-dir $RUN_DIR/instance-$i/identity --contract-params $CONTRACT_PARAMS > $RUN_DIR/instance-$i/app.log 2>&1"
```
(`bash -c "exec ..."` replaces the bash process image in place, so the spawned `os_pid` still resolves to the real game binary.) Launch them close together — this is deliberately reproducing the "multiple users start at once" race, not staggering it away. Record each instance's deskctrl PID / `os_pid`.

## Phase 4 — Watch each instance's own mainnet bootstrap
Each instance independently retries `start_embedded_node` with backoff if the mainnet refuses every gateway dial (`src/roster/connect_and_run.rs:8-13`) — this is normal and can take a while; poll each `app.log` for `embedded node ready; dial as ...` (`start_embedded_node.rs:93-98`) before expecting roster activity. Give it a few minutes; don't treat early silence as failure.

Do not `sleep`-poll via Bash — long or chained `sleep`s are blocked by the harness. Use `ScheduleWakeup` (60-180s delay) to check back on `app.log` progress across Phase 4/5, or `Monitor`'s until-loop.

## Phase 5 — Confirm convergence — and watch for the race
Poll each `app.log` (targets are `target: "roster"`) for:
- `sending roster Put` (`setup_contract.rs:141-144`) — this instance's grace window (`SETUP_CONTRACT_GRACE_SECS`, 60s) expired without finding an existing seed, so it seeded the contract.
- `roster GetResponse` / `merging own entry, sending roster Update` (`setup_contract.rs:101-121`) — this instance found and merged into an existing seed.
- **The failure mode this run exists to surface:** if *more than one* instance's log shows `sending roster Put` for this run's `$CONTRACT_PARAMS`, that's the exact disjoint-replica race described in `OBJECTIVE.md` (two concurrent first-`Put`s of a brand-new key, reconciling only via freenet-core's 5-minute InterestSync heartbeat) — call this out explicitly in the report rather than just reporting pass/fail.
- Other failure signatures to grep for: `error exited with error`, `panicked`, `update confirmation timed out`, `timeout after 30s`.

## Phase 6 — Arrange the windows, then visual confirmation to Telegram
Use deskctrl `list_windows`, matching each instance's `os_pid` (never `title` — they're identical across instances). Not optional when deskctrl is available:
1. **Tile the windows** in a two-column layout so the whole-screen video shows all N at once. The screen is 1920x1080 with a ~40px Cinnamon bottom panel, so useable height ≈ 1040. Split horizontally with `gap` = 8px: left column `Wl`, right column `Wr`, `Wl + gap + Wr = 1920` — equal halves (`Wl = Wr = 956`) give good proportions.
   - **Instance 1 = the tall left window** at `(0,0)`, size `956x1040` — it gets the full height so its UI is easy to read.
   - Right column holds instances `2..N` (`K = N−1`) stacked vertically, each `Wr` wide at `x = Wl + gap = 964`: `h_r = (1040 − (K−1)·gap) / K`; the `j`-th right instance (`j = i−1`) is at `y = (j−1)·(h_r + gap)`.
   - For N=3: `i1 956x1040 @(0,0)` · `i2 956x516 @(964,0)` · `i3 956x516 @(964,524)`.
   - Move+resize each matched window via Bash: `wmctrl -i -r <window_id> -e 0,<x_i>,<y_i>,<w_i>,<h_i>`. Re-run `list_windows` after to confirm all three are visible and non-overlapping before recording; tweak total width to ~1912 if the WM clips the right edge.
2. `screenshot` each instance's window by `window_id`, `send_to_telegram` (default `true`) posts each photo as taken; caption with the instance number.
3. `record_video` (whole screen, no `window_id`) for ~15-30s once instances start converging/moving, then `record_video stop:true` with a `summary` — posts the MP4 to Telegram. The tiled column from step 1 is what makes all N instances visible in this recording.

## Phase 7 — Tear every instance down
**Always kill the instances at the end — never leave them running.** They are real mainnet peers holding sockets and a Freenet node each; leftovers from an earlier run corrupt the next one and keep talking to the network indefinitely.

Do this *after* Phase 6 has captured its screenshots and video (those artifacts are the durable record — the live windows are not), and do it even when the run failed, errored, or converged early:
1. deskctrl `kill_process` for each recorded instance PID. This may be denied by the auto-mode classifier for some or all PIDs — that is expected, not a run failure; don't stop or retry on the denial, just move to step 2.
2. Then `pkill -f freenet-libp2p-bevy-example-1` from Bash — this is the authoritative kill and must always run regardless of whether step 1 succeeded or was denied.
3. Confirm with `pgrep -af freenet-libp2p-bevy-example-1` and report the result. Nothing should remain.

The logs under `$RUN_DIR` survive the kill — that is where the evidence lives.

## Phase 8 — Report
Compose the report below, print it to the user, and also send it via `send_to_telegram` (`text` = the same content) so it lands in the same Telegram thread as the Phase 6 screenshots/video:
- `RUN_DIR`, each instance's `app.log` path, and `CONTRACT_PARAMS` used.
- Each instance's PID/os_pid/window id.
- How many instances logged `sending roster Put` (1 = clean seed, 2+ = race reproduced — the interesting case).
- Grep cheat-sheet: `grep -E "error|panic|timed out|sending roster Put" "$RUN_DIR"/instance-*/app.log`.
- Confirmation that all instances were killed and `pgrep` came back empty.

## Hard rules
- Never launch via `cargo run`.
- Never pass `--freenet-local` or `--freenet-gateway` to any instance — the whole point is independent mainnet discovery.
- Always pass the same `--contract-params` value to every instance in a run, and always generate a fresh one per run — never reuse a previous run's value, and never omit it (omitting it means the real production contract).
- Never delete or overwrite a previous `$RUN_DIR`.
- Never end the run with game processes still alive — Phase 7 is mandatory, including on the failure path.
