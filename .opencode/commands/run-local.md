---
description: "Run the full local cross-machine pipeline on the two self-hosted runners via the test-orchestrator MCP. Default target: freenet_libp2p_bevy_example_3. Pass 'example1'/'example_1'/'ex1' to target example_1 instead. Default build profile is fast dev + tests on Linux AND Windows + cross-OS sync probes (roster, movement, engine determinism); a 'release' argument switches to a release build. Stops and asks before proceeding if the two machines share a LAN."
---

You are orchestrating the local cross-machine pipeline for the target game crate via the `test-orchestrator` MCP. Everything runs on the two self-hosted runners; you drive it with the MCP tools. `$ARGUMENTS` selects both the target crate and the build profile.

Work through these phases in order, and never skip a phase.

## Phase 0 — Determine target crate and build mode
Parse `$ARGUMENTS` into two independent choices:
- **Target crate**: contains `example1`, `example_1`, or `ex1` → **example_1**. Otherwise → **example_3** (default; `example3`/`ex3` are accepted explicitly).

| target | project dir | game binary | RUST_LOG |
|--------|-------------|-------------|----------|
| example_1 | `freenet_libp2p_bevy_example_1` | `freenet-libp2p-bevy-example-1` | `warn,roster=trace,p2p=debug,freenet_bevy=debug` |
| example_3 | `freenet_libp2p_bevy_example_3` | `freenet-libp2p-bevy-example-3` | `warn,roster=trace,p2p=debug` |

- **Build mode**: contains `release` → `MODE=release`; otherwise → `MODE=dev`. The tokens combine (`/run-local ex1 release`).
- The remaining phases refer to the selected dir as `<crate>` and its RUST_LOG from the table.

## Lead-in — Crate logging verbosity
The bevy app must be launched with the target's `RUST_LOG` from the table above (**trace on `roster`, debug on `p2p`**, plus `freenet_bevy=debug` for example_1) so the run produces the logs the cross-OS checks rely on — example_3's engine/netcode greps are `"received peer input"`, `"sending engine snapshot"`, and `"state hash"`, all emitted on the `p2p` target. The workflow exports this filter itself in the cross-os jobs; when inspecting logs, confirm the filter took effect (the app/test default is quieter: `warn,roster=info,p2p=info`).

## Phase 1 — Runner pre-flight (STOP on failure)
1. Call `list_runners`. Print both machines and their status.
2. You need BOTH the Linux runner and the Windows runner **online**.
   - If the **Linux** runner is offline → **start it yourself** with `~/actions-runner/run.sh` (you are on Linux).
   - If the **Windows** runner is offline → **STOP and tell the user the exact command to run** on Windows: `C:\actions-runner\run.cmd`. Wait for them to confirm it's up, then re-run Phase 1.
   - Do not trigger anything until both runners are online.

## Phase 2 — Same-LAN gate (STOP and ASK)
1. Call `probe_network` to get each machine's public IP + LAN IP (it triggers the `network-probe` workflow and waits).
2. Compare the two `public_ip` values (fall back to `lan_ip` / subnet if a public IP is `unknown`).
   - If they **differ** (different networks) → proceed silently; this is the intended cross-network case.
   - If they **match** (same LAN/NAT) → **STOP and ask the user**, verbatim: **"You are on the same LAN! Was this intended? Yes/No"**.
     - Answer **Yes** → proceed.
     - Answer **No** → abort the whole command and tell the user to move one machine to a different network before re-running.

## Phase 3 — Trigger the pipeline
Call `run_pipeline` with:
- `crate` = the selected project dir from Phase 0 (e.g. `freenet_libp2p_bevy_example_3`)
- `run_tests` = `true`
- `release_builds` = `true`
- `build_mode` = `MODE`

This runs: the full test gate on both Linux and Windows, the shared contract WASM build on Linux, the `MODE` binary build on both, the per-machine engine determinism gate (POLISH §3 hash record), and the cross-OS mainnet sync probes on both machines. Note: crates without a `firewall_probe/` member skip the firewall probe automatically.

## Phase 4 — Poll to completion
Poll with `run_status` (and `list_runs` for an overview) until the run is finished. Do not report prematurely. Summarize each job's result (`resolve`, `firewall-probe` (if present), `test (linux)`, `test (windows)`, `build-contract`, `build (linux)`, `build (windows)`, `cross-os-build (linux/windows)`, `cross-os-peer-discovery (linux/windows)`, `cross-os-movement-sync (linux/windows)`, `cross-os-verify`).

Note: with a single Linux runner the whole graph serialises — the Windows leg cannot start until Linux jobs free the runner, and the concurrent cross-os legs need the Linux runner idle. Expect the Windows work to be queued behind Linux jobs.

## Phase 5 — Collect the bevy logs (the investigation artifact)
1. Determine the run id of the finished pipeline.
2. `download_artifacts` (pattern `cross-os-*`, dest `<crate>/logs/<run_id>/`) so you fetch, per machine: `cross-os-log-linux` / `cross-os-log-windows` (roster JSON-lines), `cross-os-movement-log-*` (positions + remote tick), and — for example_3 — `cross-os-determinism-log-*` (the POLISH §3 final state hashes).
3. Confirm exactly one `.log` per machine landed **per category** that exists. These JSON-lines files are what the embedded cross-OS tests reported — each row is `{"machine","own","observed","t"}` (roster), `{"machine","own","t",...,"remote_x",...}` (movement), or `{"machine","final_state_hash","t"}` (determinism).

## Phase 6 — Final report
Print a concise report to the user:
- Target crate + MODE, and the job matrix with pass/fail (Phase 4 results).
- The `cross-os-verify` verdicts: roster PASS/FAIL, movement PASS/FAIL, and — for example_3 — the cross-OS determinism verdict (quote both machines' `final_state_hash`; they must be equal).
- The exact paths of the collected `.log` files under `<crate>/logs/<run_id>/`.
- If any verify step failed, quote from each log what each machine observed so the user can investigate the bug.
- Any runner issue you hit, and what the user must do manually (Windows: run `C:\actions-runner\run.cmd`; Linux: you start it yourself).

## Hard rules
- Never trigger the pipeline before Phase 1 confirms both runners online.
- Never skip the Phase 2 same-LAN question when public IPs match.
- Never use `--release` unless `$ARGUMENTS` asks for it — dev is the default and the point.
- Do not stage, commit, push, or trigger GitHub-hosted (tag) CI from this command.
